//! # Three-way merge
//!
//! Reconcile two divergent edits of a calendar against their common base.
//!
//! [`IcalMerge::merge`] is the unit a synchronisation engine needs: given a base
//! calendar and two calendars derived from it, it reports what each side changed
//! relative to the base and builds one merged calendar. Never
//! last-writer-wins: a field only one side touched is taken from that side, and
//! a field both sides touched is a conflict, reported so a caller can resolve it
//! differently. The merged calendar starts as a clone of the left one, so the
//! left side's bytes are there exactly as they were, folds included; the right
//! side's actions are then replayed line by line, so every line the right side
//! did not touch keeps its bytes too.
//!
//! ## The baseline side and the winning side
//!
//! Which side supplies the baseline and which side wins a collision are two
//! questions, and they are answered separately. The baseline is a question
//! about bytes: it decides whose folding, whose parameter casing and whose
//! property order come out untouched, so a caller answers it with the version
//! it would rather not churn. The winner is a question about policy: it decides
//! whose value survives where two people wrote different things into one field,
//! so a caller answers it with what it knows about those two people.
//! [`prefer`](IcalMerge::prefer) states the second, and left alone it keeps the
//! left side winning, as a merge has always done.
//!
//! The two have to be separable because authority sits on the replayed side. A
//! caller that wants its own edit refused where it exceeds an attendee's
//! authority has to put that edit on the right, and were the winner implied by
//! the baseline, that caller would lose every collision as the price of being
//! judged.
//!
//! ## What is matched with what
//!
//! A component is matched across the three calendars by its `UID` and its
//! `RECURRENCE-ID`, the identity iCalendar itself uses (RFC 5545 3.8.4.7,
//! 3.8.4.4): an override of one instance is never confused with the series it
//! belongs to, however the two are ordered in the file. A component carrying no
//! `UID` (a `VALARM`, a `STANDARD`, a `VTIMEZONE` observance) is matched by its
//! position among its same-named siblings.
//!
//! Inside a matched component, a property is matched by its identity where
//! iCalendar gives it one and by its position otherwise. A property that may
//! occur more than once and whose value names a thing outside the calendar is
//! identified by that value: `ATTENDEE` by its calendar user address,
//! `ATTACH` by its URI or inline binary, `RELATED-TO` by the `UID` it points
//! at, `CONFERENCE` and `IMAGE` by their URI. A different calendar address is
//! a different person, so two properties carrying different identities are
//! never matched with each other, and a value two siblings share tells neither
//! of them apart, so both of those fall back to their positions.
//!
//! Everything else is matched by name, then by equality, then by position, and
//! a position an action carries is the one its target held in the base,
//! translated through the baseline side's own removals before it is resolved
//! against the merged calendar. An addition is the exception, since it names a
//! property the base did not hold: it carries the position it holds in the
//! side that added it, and never meets an action addressed in the base.
//!
//! ## What counts as a change
//!
//! A whole property added or removed, a value changed, one item of a list value
//! added or removed, a parameter added, removed or changed. List items merge as
//! a set, both sides' additions and removals applying, so they never collide.
//!
//! ## The three ways a merge can conflict
//!
//! **Divergence.** Both sides changed the same field. The preferred side's
//! outcome is kept, the left side's unless the caller says otherwise, except
//! where a removal meets an update: there the update wins whichever side it
//! came from and whatever the preference, because keeping data beats losing it
//! silently.
//!
//! **Recurrence.** One side changed what defines the series (its `DTSTART`,
//! `DTEND`, `DURATION`, `RRULE`, `RDATE` or `EXDATE`, or the series component
//! itself) while the other changed one instance of it. Neither is wrong and
//! both survive, but a rule that moved may have moved the ground the override
//! stood on, so it is reported. A change to anything else the series carries
//! cannot have moved an occurrence and is not reported against one.
//!
//! **Authority.** An attendee may not rewrite what the organiser owns (RFC 5546
//! 3.2). Set [`right_speaks_for`](IcalMerge::right_speaks_for) to the calendar
//! address the right side edits as, and a right-side change to an
//! organiser-owned property of a component someone else organises is refused and
//! reported. Left unset, no such claim is made and nothing is refused on this
//! ground.

use core::cmp::Reverse;

use alloc::{
    borrow::{Cow, ToOwned},
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    component::IcalComponentKind,
    param::IcalParam,
    prop::{IcalPropKind, IcalPropName},
    tree::{
        cst::{IcalCst, IcalItem},
        leaf::IcalLeaf,
        line::IcalLine,
        value::cursor::IcalValueCursor,
    },
    value::IcalValue,
    version::IcalVersion,
};

/// A three-way merge waiting to run.
///
/// The three calendars, plus who the right side speaks for and which side wins
/// a collision. See the module documentation for the matching, granularity and
/// conflict rules.
pub struct IcalMerge<'m, 'a> {
    /// The common ancestor both sides were derived from.
    pub base: &'m IcalCst<'a>,
    /// One side. Its bytes are the ones the merged calendar is built from,
    /// which is a statement about bytes alone: which side wins a collision is
    /// [`prefer`](Self::prefer).
    pub left: &'m IcalCst<'a>,
    /// The other side. Its changes are replayed onto the left's bytes, and it
    /// is the only side judged for authority, whichever side is preferred.
    pub right: &'m IcalCst<'a>,
    /// The calendar address the right side edits on behalf of, when it is an
    /// attendee rather than the organiser of what it changed. Unset means no
    /// claim, and no change is refused for want of authority.
    pub right_speaks_for: Option<Cow<'a, str>>,
    /// Whose value the merged calendar carries where both sides changed one
    /// property to different things. The left side by default, which is what a
    /// merge has always done. It decides that case and no other: a property one
    /// side alone touched is still taken from that side, and an update still
    /// beats a removal whichever side it came from.
    pub prefer: IcalMergeSide,
}

/// One of the two sides of a merge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IcalMergeSide {
    /// The side the merged calendar is built from, whose untouched bytes it
    /// keeps.
    #[default]
    Left,
    /// The side whose actions are replayed onto the merged calendar, and the
    /// only side organiser authority is judged against.
    Right,
}

impl<'a> IcalMerge<'_, 'a> {
    /// Run the merge.
    pub fn merge(self) -> IcalMergeReport<'a> {
        let version = self.base.version();

        let base = nodes(self.base);
        let left = nodes(self.left);
        let right = nodes(self.right);

        let left_ops = diff(&base, &left, version);
        let right_ops = diff(&base, &right, version);

        let mut merged = self.left.clone();
        let mut conflicts = Vec::new();
        let mut applicable = Vec::new();

        for op in &right_ops {
            let verdict = self.judge(op, &left_ops, &right_ops, &base, &left);

            if verdict.applies {
                applicable.push((op, verdict.displaces));
            }

            if let Some(reason) = verdict.reason {
                conflicts.push(IcalMergeConflict {
                    right: op.action.clone(),
                    reason,
                });
            }
        }

        applicable.sort_by_key(|(op, _)| replay_order(op));

        let shift = Shift::of(&left_ops);

        for (op, displaces) in applicable {
            apply(&mut merged, op, displaces, self.right, &shift);
        }

        IcalMergeReport {
            merged,
            left: left_ops.into_iter().map(|op| op.action).collect(),
            right: right_ops.into_iter().map(|op| op.action).collect(),
            conflicts,
        }
    }

    /// Whether a right-side action applies, and what to report about it.
    fn judge<'o>(
        &self,
        op: &Op<'a>,
        left_ops: &'o [Op<'a>],
        right_ops: &[Op<'a>],
        base: &[Node<'_, 'a>],
        left: &[Node<'_, 'a>],
    ) -> Verdict<'o, 'a> {
        if let Some(speaker) = &self.right_speaks_for
            && op.organiser_owned
            && organiser_of(op.path(), base, left).is_some_and(|held| held != *speaker)
        {
            return Verdict {
                applies: false,
                reason: Some(IcalMergeReason::Authority),
                displaces: None,
            };
        }

        if let Some(collision) = left_ops.iter().find(|left| self.collides(left, op)) {
            // NOTE: A removal against an update is not a stand-off: one side
            // says the data is gone and the other says what it now is. The
            // update survives whichever side it came from, since keeping data
            // beats losing it silently, which is not the caller's to invert.
            // The preference decides the rest, where both sides wrote a value.
            let applies = if scraps(collision, op) {
                true
            } else if scraps(op, collision) {
                false
            } else {
                self.prefer == IcalMergeSide::Right
            };

            return Verdict {
                applies,
                reason: Some(IcalMergeReason::Divergent(collision.action.clone())),
                displaces: (applies && both_added(collision, op)).then_some(collision),
            };
        }

        Verdict {
            // NOTE: The merged calendar is the left side, so an act the left
            // side already performed identically needs no replaying, and
            // replaying an addition would put it there twice.
            applies: !left_ops.iter().any(|held| self.agrees(held, op)),
            // NOTE: A recurrence conflict refuses nothing. Both sides said
            // something true about different parts of one series, and the
            // caller is told only because one may have moved the ground the
            // other stood on. A pair the replayed side made in full is one
            // person's own edit seen twice rather than two people disagreeing.
            reason: left_ops
                .iter()
                .find(|left| {
                    across_the_series(left, op)
                        && !right_ops.iter().any(|held| self.agrees(left, held))
                })
                .map(|left| IcalMergeReason::Recurrence(left.action.clone())),
            displaces: None,
        }
    }

    /// Whether two actions collide on one field.
    fn collides(&self, left: &Op<'a>, right: &Op<'a>) -> bool {
        // NOTE: Two people who wrote the same thing are not two people
        // disagreeing, so an identical act on both sides is no collision.
        if self.agrees(left, right) {
            return false;
        }

        match (&left.slot, &right.slot) {
            (Slot::Component, Slot::Component) => reaches(left, right) || reaches(right, left),
            (Slot::Component, _) => reaches(left, right),
            (_, Slot::Component) => reaches(right, left),
            _ if !same_prop(left.prop(), right.prop()) => false,
            // NOTE: An addition names a property the base did not hold, and is
            // addressed by a position in the side that wrote it; every other
            // action names one the base held, addressed by its position there.
            // The two numbering systems never name one property.
            _ if added(left) != added(right) => false,
            // NOTE: A property one side removed is a property the other side
            // cannot usefully edit, so a change to its value, to one of its
            // parameters or to one of its list items meets the removal.
            (Slot::Prop, _) | (_, Slot::Prop) => true,
            (Slot::Items, _) | (_, Slot::Items) => false,
            (
                Slot::Param {
                    name: left,
                    at: one,
                },
                Slot::Param {
                    name: right,
                    at: two,
                },
            ) => left == right && one == two,
            (Slot::Param { .. }, _) | (_, Slot::Param { .. }) => false,
            _ => true,
        }
    }

    /// Whether the two sides performed the same act.
    ///
    /// An addition names where it lands and what it says, never how it is
    /// spelt, so the two sides' own bytes settle it: a property and a component
    /// both sides added are the same addition only where both wrote the same
    /// thing.
    fn agrees(&self, left: &Op<'a>, right: &Op<'a>) -> bool {
        if left.action != right.action {
            return false;
        }

        match &right.action {
            IcalMergeAction::PropAdded { .. } => {
                let held = self.added_line(self.left, left);

                held.is_some() && held == self.added_line(self.right, right)
            }
            IcalMergeAction::ComponentAdded { at } => {
                let held = find(self.left, at).map(IcalCst::to_bytes);

                held.is_some() && held == find(self.right, at).map(IcalCst::to_bytes)
            }
            _ => true,
        }
    }

    /// The bytes of the line an addition put in one side.
    fn added_line(&self, cst: &IcalCst<'a>, op: &Op<'a>) -> Option<Vec<u8>> {
        let at = op.source.as_ref()?;
        let line = line_at(find(cst, &at.component)?, at, Some(at.index))?;
        let mut out = Vec::new();

        line.write_bytes(&mut out);

        Some(out)
    }
}

/// Whether an action puts a property the base did not hold.
fn added(op: &Op<'_>) -> bool {
    matches!(op.action, IcalMergeAction::PropAdded { .. })
}

/// Whether one action takes away what the other one still works on.
///
/// Granularity is what settles it rather than the word removal: a side that
/// drops one parameter of a property keeps the property, so against a side that
/// removed the property whole it is the one preserving data. Two actions at one
/// granularity are a stand-off unless exactly one of them removes.
fn scraps(one: &Op<'_>, two: &Op<'_>) -> bool {
    if !one.action.is_removal() {
        return false;
    }

    match (&one.slot, &two.slot) {
        (Slot::Component, Slot::Component) | (Slot::Prop, Slot::Prop) => !two.action.is_removal(),
        (Slot::Component, _) | (Slot::Prop, _) => true,
        _ => !two.action.is_removal(),
    }
}

/// Whether two colliding actions both add the same kind of thing, so the one
/// that wins replaces the one it beat rather than joining it.
fn both_added(left: &Op<'_>, right: &Op<'_>) -> bool {
    matches!(
        (&left.action, &right.action),
        (
            IcalMergeAction::PropAdded { .. },
            IcalMergeAction::PropAdded { .. }
        ) | (
            IcalMergeAction::ComponentAdded { .. },
            IcalMergeAction::ComponentAdded { .. }
        )
    )
}

/// Where an action sits in the order the replay applies them.
///
/// A component and a property carrying no identity of its own are addressed by
/// the position they held in the base, and taking one out renumbers every
/// same-named one after it. Removals therefore go last, highest position first,
/// so each one still names in the merged calendar what it named in the base.
/// Everything else keeps the order the diff produced, which a stable sort
/// preserves.
fn replay_order(op: &Op<'_>) -> (u8, Reverse<usize>) {
    let last = match &op.action {
        IcalMergeAction::ComponentRemoved { at } => {
            at.0.last()
                .and_then(|step| step.key.parse().ok())
                .unwrap_or(0)
        }
        IcalMergeAction::PropRemoved { at, .. } => at.index,
        _ => return (0, Reverse(0)),
    };

    (1, Reverse(last))
}

/// What a merge decided about one right-side action.
struct Verdict<'o, 'a> {
    /// Whether the action lands in the merged calendar.
    applies: bool,
    /// What to report about it, if anything.
    reason: Option<IcalMergeReason<'a>>,
    /// The left-side addition this one beat, whose property or component
    /// leaves the merged calendar so the winner replaces it rather than
    /// joining it.
    displaces: Option<&'o Op<'a>>,
}

/// How many members the baseline side took out of each group of same-named
/// properties, so a position measured in the base still names its own target
/// in the merged calendar.
struct Shift<'a>(Vec<(&'a IcalComponentPath<'a>, String, usize)>);

impl<'a> Shift<'a> {
    /// Read it off the baseline side's own removals.
    fn of(ops: &'a [Op<'a>]) -> Self {
        Self(
            ops.iter()
                .filter_map(|op| match &op.action {
                    IcalMergeAction::PropRemoved { at, .. } => {
                        Some((&at.component, at.name.to_ascii_uppercase(), at.index))
                    }
                    _ => None,
                })
                .collect(),
        )
    }

    /// Where a base position sits in the merged calendar, or `None` where the
    /// baseline side removed the very property the position names.
    fn translate(&self, at: &IcalPropPath<'_>) -> Option<usize> {
        let name = at.name.to_ascii_uppercase();
        let mut shift = 0;

        for (component, held, index) in &self.0 {
            if **component != at.component || *held != name {
                continue;
            }

            if *index == at.index {
                return None;
            }

            if *index < at.index {
                shift += 1;
            }
        }

        Some(at.index - shift)
    }
}

/// The outcome of a three-way merge.
#[derive(Clone, Debug)]
pub struct IcalMergeReport<'a> {
    /// The merged calendar: the left one with the right side's applicable
    /// actions replayed onto it, line by line.
    pub merged: IcalCst<'a>,
    /// What the left calendar changed relative to the base.
    pub left: Vec<IcalMergeAction<'a>>,
    /// What the right calendar changed relative to the base.
    pub right: Vec<IcalMergeAction<'a>>,
    /// The right-side actions that did not simply apply, and why.
    pub conflicts: Vec<IcalMergeConflict<'a>>,
}

/// A right-side action that did not simply apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalMergeConflict<'a> {
    /// The action the right side wanted.
    pub right: IcalMergeAction<'a>,
    /// Why it did not simply apply.
    pub reason: IcalMergeReason<'a>,
}

/// Why a right-side action did not simply apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalMergeReason<'a> {
    /// Both sides changed the same field. The merged calendar holds the
    /// preferred side's outcome, the left side's unless the caller said
    /// otherwise, except where a removal met an update, in which case the
    /// update was kept whichever side it came from. The action carried here is
    /// the left side's, beside the right side's on the conflict itself.
    Divergent(IcalMergeAction<'a>),
    /// One side changed a series and the other changed one of its instances.
    /// Both survive in the merged calendar; a rule that moved may have moved
    /// the ground the override stood on, which is why this is said out loud.
    Recurrence(IcalMergeAction<'a>),
    /// The right side does not speak for the organiser of the component, and
    /// the property is the organiser's to set (RFC 5546 3.2).
    Authority,
}

/// One component's address: the steps from the calendar root down to it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalComponentPath<'a>(pub Vec<IcalComponentStep<'a>>);

/// One step of a component path: a name and the identity that tells it from
/// its same-named siblings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalComponentStep<'a> {
    /// The component name, uppercase.
    pub name: Cow<'a, str>,
    /// The `UID`, with the `RECURRENCE-ID` after a solidus when the component
    /// overrides one instance; the position among same-named siblings when the
    /// component carries no `UID`.
    pub key: Cow<'a, str>,
}

/// One property's address: the component holding it, its name, and what tells
/// it from the component's other properties of that name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalPropPath<'a> {
    /// The component the property belongs to.
    pub component: IcalComponentPath<'a>,
    /// The property name as written.
    pub name: Cow<'a, str>,
    /// The position among the component's properties of that name, counted in
    /// the calendar the action was read from.
    pub index: usize,
    /// The value that tells the property from its same-named siblings, where
    /// iCalendar gives it one: the calendar user address of an `ATTENDEE`, the
    /// URI or inline binary of an `ATTACH`, the `UID` a `RELATED-TO` points
    /// at, the URI of a `CONFERENCE` or an `IMAGE`. `None` for every other
    /// property, whose position is then what tells it from its siblings, and
    /// `None` too for a value a same-named sibling repeats, which tells
    /// neither of them apart.
    pub identity: Option<Cow<'a, str>>,
}

/// One change a side made relative to the base.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalMergeAction<'a> {
    /// A component the side added.
    ComponentAdded {
        /// Where it was added.
        at: IcalComponentPath<'a>,
    },
    /// A component the side removed.
    ComponentRemoved {
        /// What it removed.
        at: IcalComponentPath<'a>,
    },
    /// A property the side added.
    PropAdded {
        /// Where it was added.
        at: IcalPropPath<'a>,
        /// The added value.
        value: IcalValue<'a>,
    },
    /// A property the side removed.
    PropRemoved {
        /// What it removed.
        at: IcalPropPath<'a>,
        /// The removed value.
        value: IcalValue<'a>,
    },
    /// A matched property whose value changed.
    ValueChanged {
        /// The changed property.
        at: IcalPropPath<'a>,
        /// The base value.
        old: IcalValue<'a>,
        /// The changed value.
        new: IcalValue<'a>,
    },
    /// One item joined a list value (`CATEGORIES`, `RDATE`, `EXDATE`).
    ValueItemAdded {
        /// The changed property.
        at: IcalPropPath<'a>,
        /// The added item.
        item: Cow<'a, str>,
    },
    /// One item left a list value.
    ValueItemRemoved {
        /// The changed property.
        at: IcalPropPath<'a>,
        /// The removed item.
        item: Cow<'a, str>,
    },
    /// A parameter the side added.
    ParamAdded {
        /// The changed property.
        at: IcalPropPath<'a>,
        /// The added parameter.
        param: IcalParam<'a>,
    },
    /// A parameter the side removed.
    ParamRemoved {
        /// The changed property.
        at: IcalPropPath<'a>,
        /// The removed parameter.
        param: IcalParam<'a>,
    },
    /// A parameter whose value changed.
    ParamChanged {
        /// The changed property.
        at: IcalPropPath<'a>,
        /// The base parameter.
        old: IcalParam<'a>,
        /// The changed parameter.
        new: IcalParam<'a>,
    },
}

impl IcalMergeAction<'_> {
    /// Whether the action takes something away.
    fn is_removal(&self) -> bool {
        matches!(
            self,
            Self::ComponentRemoved { .. }
                | Self::PropRemoved { .. }
                | Self::ValueItemRemoved { .. }
                | Self::ParamRemoved { .. }
        )
    }
}

/// One diffed change, with what the merge needs to route and judge it.
struct Op<'a> {
    /// The change itself.
    action: IcalMergeAction<'a>,
    /// Where the line carrying the change sits in the side that wrote it,
    /// which is where the replay reads its new bytes from. `None` for a
    /// removal and for a whole component.
    source: Option<IcalPropPath<'a>>,
    /// The field it occupies, at which two sides collide.
    slot: Slot,
    /// Whether the property is one only the organiser may set.
    organiser_owned: bool,
}

impl<'a> Op<'a> {
    /// The component the action lands in.
    fn path(&self) -> &IcalComponentPath<'a> {
        match &self.action {
            IcalMergeAction::ComponentAdded { at } | IcalMergeAction::ComponentRemoved { at } => at,
            IcalMergeAction::PropAdded { at, .. }
            | IcalMergeAction::PropRemoved { at, .. }
            | IcalMergeAction::ValueChanged { at, .. }
            | IcalMergeAction::ValueItemAdded { at, .. }
            | IcalMergeAction::ValueItemRemoved { at, .. }
            | IcalMergeAction::ParamAdded { at, .. }
            | IcalMergeAction::ParamRemoved { at, .. }
            | IcalMergeAction::ParamChanged { at, .. } => &at.component,
        }
    }

    /// The property the action lands on, for the actions that have one.
    fn prop(&self) -> Option<&IcalPropPath<'a>> {
        match &self.action {
            IcalMergeAction::ComponentAdded { .. } | IcalMergeAction::ComponentRemoved { .. } => {
                None
            }
            IcalMergeAction::PropAdded { at, .. }
            | IcalMergeAction::PropRemoved { at, .. }
            | IcalMergeAction::ValueChanged { at, .. }
            | IcalMergeAction::ValueItemAdded { at, .. }
            | IcalMergeAction::ValueItemRemoved { at, .. }
            | IcalMergeAction::ParamAdded { at, .. }
            | IcalMergeAction::ParamRemoved { at, .. }
            | IcalMergeAction::ParamChanged { at, .. } => Some(at),
        }
    }
}

/// The field of a property an action occupies.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Slot {
    /// The whole component.
    Component,
    /// The whole property.
    Prop,
    /// The whole value.
    Value,
    /// The items of a list value, which merge as a set and never collide.
    Items,
    /// One parameter, by name and by its position among the property's
    /// parameters of that name.
    Param {
        /// The parameter name, uppercase.
        name: String,
        /// The position among the property's parameters of that name.
        at: usize,
    },
}

/// Whether a component-level action takes away or replaces what another action
/// works on.
///
/// A component one side removed or added is a component the other side cannot
/// usefully edit, at any depth. Two removals overlapping are left alone: both
/// sides agreed the data goes, and saying so would be noise.
fn reaches(above: &Op<'_>, below: &Op<'_>) -> bool {
    below.path().0.starts_with(&above.path().0)
        && (above.path() == below.path() || !below.action.is_removal())
}

/// Whether two actions address one property: the same component, the same
/// name, and the same identity where the property has one or the same position
/// where it has not.
fn same_prop(left: Option<&IcalPropPath<'_>>, right: Option<&IcalPropPath<'_>>) -> bool {
    let (Some(left), Some(right)) = (left, right) else {
        return false;
    };

    if left.component != right.component || !left.name.eq_ignore_ascii_case(&right.name) {
        return false;
    }

    // NOTE: A property with an identity is never the one without: where one
    // side repeats a value and the other does not, the two are told apart
    // differently, and a position on one side does not answer for an identity
    // on the other.
    match (&left.identity, &right.identity) {
        (Some(left), Some(right)) => left == right,
        (None, None) => left.index == right.index,
        _ => false,
    }
}

/// Whether one action changed what defines a series and the other one of its
/// instances.
fn across_the_series(left: &Op<'_>, right: &Op<'_>) -> bool {
    let (Some(one), Some(two)) = (left.path().0.last(), right.path().0.last()) else {
        return false;
    };

    let (Some(left_uid), Some(right_uid)) = (one.key.split('/').next(), two.key.split('/').next())
    else {
        return false;
    };

    // NOTE: Same UID, and exactly one of the two carries a RECURRENCE-ID: one
    // side is talking about the whole series and the other about one of its
    // occurrences.
    if one.name != two.name
        || left_uid != right_uid
        || one.key.contains('/') == two.key.contains('/')
    {
        return false;
    }

    let series = if one.key.contains('/') { right } else { left };

    defines_the_set(series)
}

/// Whether an action changed what a recurrence set is made of, rather than
/// something the series merely describes (RFC 5545 3.8.5).
fn defines_the_set(op: &Op<'_>) -> bool {
    let Some(at) = op.prop() else {
        return true;
    };

    matches!(
        IcalPropName::from(Cow::Owned(at.name.to_ascii_uppercase())),
        IcalPropName::Kind(
            IcalPropKind::DtStart
                | IcalPropKind::DtEnd
                | IcalPropKind::Duration
                | IcalPropKind::RRule
                | IcalPropKind::RDate
                | IcalPropKind::ExDate
        )
    )
}

/// One component of a calendar, with the path that addresses it.
struct Node<'c, 'a> {
    /// The path from the calendar root.
    path: IcalComponentPath<'a>,
    /// The component itself.
    cst: &'c IcalCst<'a>,
}

/// Every component of a calendar, the root first, each with its path.
fn nodes<'c, 'a>(cst: &'c IcalCst<'a>) -> Vec<Node<'c, 'a>> {
    let mut out = Vec::new();
    walk(cst, IcalComponentPath::default(), &mut out);
    out
}

/// Collect one component and everything nested in it.
fn walk<'c, 'a>(cst: &'c IcalCst<'a>, path: IcalComponentPath<'a>, out: &mut Vec<Node<'c, 'a>>) {
    out.push(Node {
        path: path.clone(),
        cst,
    });

    let mut seen: Vec<(String, usize)> = Vec::new();

    for child in components(cst) {
        let name = component_name(child);
        let ordinal = match seen.iter_mut().find(|(held, _)| *held == name) {
            Some((_, count)) => {
                *count += 1;
                *count
            }
            None => {
                seen.push((name.clone(), 0));
                0
            }
        };

        let mut nested = path.clone();
        nested.0.push(IcalComponentStep {
            key: Cow::Owned(key(child, ordinal)),
            name: Cow::Owned(name),
        });

        walk(child, nested, out);
    }
}

/// The components nested directly in one.
fn components<'c, 'a>(cst: &'c IcalCst<'a>) -> impl Iterator<Item = &'c IcalCst<'a>> {
    cst.items.iter().filter_map(|item| match item {
        IcalItem::Component(child) => Some(&**child),
        _ => None,
    })
}

/// A component's name, uppercase.
fn component_name(cst: &IcalCst<'_>) -> String {
    cst.begin
        .as_ref()
        .map(|begin| begin.raw_value_str().to_ascii_uppercase())
        .unwrap_or_default()
}

/// A component's identity among its same-named siblings: its `UID`, with the
/// `RECURRENCE-ID` after a solidus when it overrides one instance, or its
/// position when it carries no `UID`.
fn key(cst: &IcalCst<'_>, ordinal: usize) -> String {
    let Some(uid) = raw(cst, IcalPropKind::Uid) else {
        return ordinal.to_string();
    };

    match raw(cst, IcalPropKind::RecurrenceId) {
        Some(id) => format!("{uid}/{id}"),
        None => uid,
    }
}

/// The raw text of a component's first property of this kind.
fn raw(cst: &IcalCst<'_>, kind: IcalPropKind) -> Option<String> {
    lines(cst)
        .find(|line| line.name.get().eq_ignore_ascii_case(&kind))
        .map(|line| line.raw_value_str().into_owned())
}

/// The property lines of a component, in source order.
///
/// `BEGIN` and `END` are the component envelope rather than properties, and a
/// bare, envelope-less record holds them as lines like any other, so they are
/// skipped here: no side is ever reported as having added or removed one, and
/// none is ever copied into a calendar that would then refuse to parse.
fn lines<'c, 'a>(cst: &'c IcalCst<'a>) -> impl Iterator<Item = &'c IcalLine<'a>> {
    cst.items
        .iter()
        .filter_map(|item| match item {
            IcalItem::Prop(line) => Some(line),
            _ => None,
        })
        .filter(|line| !structural(line))
}

/// Whether a line is a component envelope keyword rather than a property.
fn structural(line: &IcalLine<'_>) -> bool {
    let name = line.name.get();

    name.eq_ignore_ascii_case("BEGIN") || name.eq_ignore_ascii_case("END")
}

/// The calendar address organising the component an action lands in, read from
/// the base and, failing that, from the left side.
fn organiser_of<'a>(
    path: &IcalComponentPath<'a>,
    base: &[Node<'_, 'a>],
    left: &[Node<'_, 'a>],
) -> Option<String> {
    base.iter()
        .chain(left)
        .find(|node| node.path == *path)
        .and_then(|node| raw(node.cst, IcalPropKind::Organizer))
}

/// Whether a whole component is the organiser's to add or remove.
///
/// An attendee sets their own alarms on a meeting they were invited to; the
/// meeting itself is the organiser's.
fn whole_component_owned(path: &IcalComponentPath<'_>) -> bool {
    !path
        .0
        .last()
        .is_some_and(|step| matches!(step.name.parse(), Ok(IcalComponentKind::VAlarm)))
}

/// Whether a property of a scheduled component is one only its organiser may
/// set (RFC 5546 3.2).
///
/// An attendee owns their own `ATTENDEE` line, the transparency they show to
/// others, their alarms, and anything outside the vocabulary. Everything that
/// describes the meeting itself is the organiser's.
fn organiser_owned(component: &IcalComponentPath<'_>, name: &IcalPropName<'_>) -> bool {
    let scheduled = component.0.last().is_some_and(|step| {
        matches!(
            step.name.parse(),
            Ok(IcalComponentKind::VEvent | IcalComponentKind::VTodo | IcalComponentKind::VJournal)
        )
    });

    let IcalPropName::Kind(kind) = name else {
        return false;
    };

    scheduled
        && !matches!(
            kind,
            IcalPropKind::Attendee | IcalPropKind::Transp | IcalPropKind::DtStamp
        )
}

/// Diff one side against the base: one op per observed change.
fn diff<'a>(base: &[Node<'_, 'a>], side: &[Node<'_, 'a>], version: IcalVersion) -> Vec<Op<'a>> {
    let mut ops = Vec::new();

    for node in base {
        if !side.iter().any(|held| held.path == node.path) && !removed_above(&node.path, side, base)
        {
            ops.push(Op {
                action: IcalMergeAction::ComponentRemoved {
                    at: node.path.clone(),
                },
                source: None,
                slot: Slot::Component,
                organiser_owned: whole_component_owned(&node.path),
            });
        }
    }

    for node in side {
        if !base.iter().any(|held| held.path == node.path) && !added_above(&node.path, side, base) {
            ops.push(Op {
                action: IcalMergeAction::ComponentAdded {
                    at: node.path.clone(),
                },
                source: None,
                slot: Slot::Component,
                organiser_owned: whole_component_owned(&node.path),
            });
        }
    }

    // NOTE: A calendar may hold two components at one path, a `UID` written
    // twice with no `RECURRENCE-ID` telling them apart, so each side component
    // is matched once: matching both base components against the same one
    // would report the difference between them as a change either side made.
    let mut taken = alloc::vec![false; side.len()];

    for node in base {
        let Some((at, held)) = side
            .iter()
            .enumerate()
            .find(|(at, held)| !taken[*at] && held.path == node.path)
        else {
            continue;
        };

        taken[at] = true;

        diff_component(node, held, version, &mut ops);
    }

    ops
}

/// Whether an ancestor of this path is itself missing from the side, so the
/// removal is already reported one level up.
fn removed_above(
    path: &IcalComponentPath<'_>,
    side: &[Node<'_, '_>],
    base: &[Node<'_, '_>],
) -> bool {
    ancestors(path).any(|above| {
        base.iter().any(|node| node.path == above) && !side.iter().any(|node| node.path == above)
    })
}

/// The mirror of [`removed_above`] for an addition.
fn added_above(path: &IcalComponentPath<'_>, side: &[Node<'_, '_>], base: &[Node<'_, '_>]) -> bool {
    ancestors(path).any(|above| {
        side.iter().any(|node| node.path == above) && !base.iter().any(|node| node.path == above)
    })
}

/// Every proper ancestor path of a path, nearest first.
fn ancestors<'p, 'a>(
    path: &'p IcalComponentPath<'a>,
) -> impl Iterator<Item = IcalComponentPath<'a>> + 'p {
    (1..path.0.len()).map(|depth| IcalComponentPath(path.0[..depth].to_vec()))
}

/// Diff the properties of one matched component pair.
fn diff_component<'a>(
    base: &Node<'_, 'a>,
    side: &Node<'_, 'a>,
    version: IcalVersion,
    ops: &mut Vec<Op<'a>>,
) {
    let base_props: Vec<&IcalLine<'a>> = lines(base.cst).collect();
    let side_props: Vec<&IcalLine<'a>> = lines(side.cst).collect();

    let mut names: Vec<String> = Vec::new();
    for line in base_props.iter().chain(&side_props) {
        let name = line.name.get().to_ascii_uppercase();
        if !names.contains(&name) {
            names.push(name);
        }
    }

    for name in names {
        let of = |lines: &[&IcalLine<'a>]| -> Vec<usize> {
            lines
                .iter()
                .enumerate()
                .filter(|(_, line)| line.name.get().eq_ignore_ascii_case(&name))
                .map(|(index, _)| index)
                .collect()
        };

        let mut base_free = of(&base_props);
        let mut side_free = of(&side_props);

        // NOTE: An untouched property pairs with itself before anything else is
        // consulted, so adding one line does not renumber every line after it.
        let mut pairs = Vec::new();
        let mut b = 0;
        while b < base_free.len() {
            let same = side_free.iter().position(|&s| {
                base_props[base_free[b]].decode(version) == side_props[s].decode(version)
            });

            match same {
                Some(s) => pairs.push((base_free.remove(b), side_free.remove(s))),
                None => b += 1,
            }
        }

        let mut b = 0;
        while b < base_free.len() {
            let held = identity_in(&base_props, base_free[b]);
            let same = held.and_then(|held| {
                side_free
                    .iter()
                    .position(|&s| identity_in(&side_props, s).is_some_and(|side| side == held))
            });

            match same {
                Some(s) => pairs.push((base_free.remove(b), side_free.remove(s))),
                None => b += 1,
            }
        }

        // NOTE: Position only tells apart properties iCalendar gives no
        // identity of their own. A calendar address that matched nothing names
        // a person who left, never a person the other side renamed.
        let mut b = 0;
        while b < base_free.len() {
            if identity_in(&base_props, base_free[b]).is_some() {
                b += 1;
                continue;
            }

            let same = side_free
                .iter()
                .position(|&s| identity_in(&side_props, s).is_none());

            match same {
                Some(s) => pairs.push((base_free.remove(b), side_free.remove(s))),
                None => break,
            }
        }

        for index in base_free {
            let line = base_props[index];
            let at = prop_path(&base.path, &base_props, index);

            ops.push(Op {
                organiser_owned: organiser_owned(&base.path, &decode_name(line)),
                action: IcalMergeAction::PropRemoved {
                    value: line.decode(version).value.into_owned(),
                    at,
                },
                source: None,
                slot: Slot::Prop,
            });
        }

        for index in side_free {
            let line = side_props[index];
            let at = prop_path(&side.path, &side_props, index);

            ops.push(Op {
                organiser_owned: organiser_owned(&side.path, &decode_name(line)),
                action: IcalMergeAction::PropAdded {
                    value: line.decode(version).value.into_owned(),
                    at: at.clone(),
                },
                source: Some(at),
                slot: Slot::Prop,
            });
        }

        for (b, s) in pairs {
            diff_prop(&base.path, &base_props, b, &side_props, s, version, ops);
        }
    }
}

/// The value that tells a property from its same-named siblings, where
/// iCalendar gives it one.
///
/// A property that may occur more than once in a component and whose value
/// names a thing outside the calendar is that thing: an `ATTENDEE` is a
/// calendar user address (RFC 5545 3.8.4.1), an `ATTACH` a URI or an inline
/// binary (3.8.1.1), a `RELATED-TO` the `UID` of another component (3.8.4.5), a
/// `CONFERENCE` and an `IMAGE` a URI (RFC 7986 5.11, 5.10). Every other
/// property has none: either it may occur only once, so its name already tells
/// it apart, or its value is the datum being edited, and keying on it would
/// make every edit a replacement.
/// An identity that does not tell a property from its same-named siblings is
/// no identity: a value written twice in one component names both of them, so
/// those fall back to their positions, and a sibling that is still alone with
/// its value keeps its own.
fn identity_in<'a>(lines: &[&IcalLine<'a>], at: usize) -> Option<Cow<'a, str>> {
    let held = identity_of(lines[at])?;
    let name = lines[at].name.get();

    let twice = lines.iter().enumerate().any(|(index, line)| {
        index != at
            && line.name.get().eq_ignore_ascii_case(name)
            && identity_of(line).is_some_and(|line| line == held)
    });

    (!twice).then_some(held)
}

/// The identity a property name carries, read off one line.
fn identity_of<'a>(line: &IcalLine<'a>) -> Option<Cow<'a, str>> {
    let identified = matches!(
        decode_name(line),
        IcalPropName::Kind(
            IcalPropKind::Attendee
                | IcalPropKind::Attach
                | IcalPropKind::RelatedTo
                | IcalPropKind::Conference
                | IcalPropKind::Image
        )
    );

    identified.then(|| Cow::Owned(value_text(line)))
}

/// The whole raw value of a line, as written.
///
/// Not its first value: a `CAL-ADDRESS` list is one value in the merge's eyes,
/// and reading only up to the first comma would give two different lines one
/// identity.
fn value_text(line: &IcalLine<'_>) -> String {
    let mut out = Vec::new();

    line.value.write_bytes(&mut out);

    String::from_utf8_lossy(&out).into_owned()
}

/// The name a line decodes to.
fn decode_name<'a>(line: &IcalLine<'a>) -> IcalPropName<'a> {
    IcalPropName::from(Cow::Owned(line.name.get().to_owned()))
}

/// What tells a line from its component's other properties of that name: its
/// identity where it has one, and its position either way.
fn prop_path<'a>(
    component: &IcalComponentPath<'a>,
    lines: &[&IcalLine<'a>],
    at: usize,
) -> IcalPropPath<'a> {
    let name = lines[at].name.get();
    let index = lines[..at]
        .iter()
        .filter(|held| held.name.get().eq_ignore_ascii_case(name))
        .count();

    IcalPropPath {
        component: component.clone(),
        name: Cow::Owned(name.to_owned()),
        index,
        identity: identity_in(lines, at),
    }
}

/// Diff one matched property pair: its parameters, then its value.
#[allow(clippy::too_many_arguments)]
fn diff_prop<'a>(
    component: &IcalComponentPath<'a>,
    lines: &[&IcalLine<'a>],
    at: usize,
    side_lines: &[&IcalLine<'a>],
    side_at: usize,
    version: IcalVersion,
    ops: &mut Vec<Op<'a>>,
) {
    let base = lines[at];
    let side = side_lines[side_at];
    let at = prop_path(component, lines, at);
    let source = prop_path(component, side_lines, side_at);
    let owned = organiser_owned(component, &decode_name(base));

    let base_prop = base.decode(version);
    let side_prop = side.decode(version);

    for (index, param) in base_prop.params.iter().enumerate() {
        let name = param_name(param);
        let ordinal = ordinal_of(&base_prop.params, index, &name);
        let held = nth_param(&side_prop.params, &name, ordinal);

        let action = match held {
            None => IcalMergeAction::ParamRemoved {
                at: at.clone(),
                param: param.clone().into_owned(),
            },
            Some(held) if held != param => IcalMergeAction::ParamChanged {
                at: at.clone(),
                old: param.clone().into_owned(),
                new: held.clone().into_owned(),
            },
            Some(_) => continue,
        };

        ops.push(Op {
            action,
            source: Some(source.clone()),
            slot: Slot::Param { name, at: ordinal },
            organiser_owned: owned,
        });
    }

    for (index, param) in side_prop.params.iter().enumerate() {
        let name = param_name(param);
        let ordinal = ordinal_of(&side_prop.params, index, &name);

        if nth_param(&base_prop.params, &name, ordinal).is_some() {
            continue;
        }

        ops.push(Op {
            action: IcalMergeAction::ParamAdded {
                at: at.clone(),
                param: param.clone().into_owned(),
            },
            source: Some(source.clone()),
            slot: Slot::Param { name, at: ordinal },
            organiser_owned: owned,
        });
    }

    // NOTE: The decoded values are what is compared, not the raw bytes: a line
    // that was rewritten without changing what it says has not changed, and the
    // merged calendar keeps the left side's spelling of it either way.
    if base_prop.value == side_prop.value {
        return;
    }

    match (&base_prop.value, &side_prop.value) {
        // NOTE: A list is a set: both sides' additions and both sides'
        // removals apply, so two sides editing one list never collide.
        (IcalValue::TextList(old), IcalValue::TextList(new)) => {
            list_ops(&at, &source, &old.0, &new.0, owned, ops)
        }
        (IcalValue::DateTimeList(old), IcalValue::DateTimeList(new)) => {
            list_ops(&at, &source, &old.0, &new.0, owned, ops)
        }
        (old, new) => ops.push(Op {
            action: IcalMergeAction::ValueChanged {
                at,
                old: old.clone().into_owned(),
                new: new.clone().into_owned(),
            },
            source: Some(source),
            slot: Slot::Value,
            organiser_owned: owned,
        }),
    }
}

/// The item-by-item difference between two list values.
fn list_ops<'a>(
    at: &IcalPropPath<'a>,
    source: &IcalPropPath<'a>,
    old: &[Cow<'_, str>],
    new: &[Cow<'_, str>],
    owned: bool,
    ops: &mut Vec<Op<'a>>,
) {
    let removed = old.iter().filter(|item| !new.contains(item));
    let added = new.iter().filter(|item| !old.contains(item));

    for item in removed {
        ops.push(Op {
            action: IcalMergeAction::ValueItemRemoved {
                at: at.clone(),
                item: Cow::Owned(item.to_string()),
            },
            source: Some(source.clone()),
            slot: Slot::Items,
            organiser_owned: owned,
        });
    }

    for item in added {
        ops.push(Op {
            action: IcalMergeAction::ValueItemAdded {
                at: at.clone(),
                item: Cow::Owned(item.to_string()),
            },
            source: Some(source.clone()),
            slot: Slot::Items,
            organiser_owned: owned,
        });
    }
}

/// A parameter's name, the key two sides' parameters are matched on.
fn param_name(param: &IcalParam<'_>) -> String {
    match param {
        IcalParam::Unknown { name, .. } => name.to_ascii_uppercase(),
        known => known
            .kind()
            .map(|kind| kind.to_ascii_uppercase())
            .unwrap_or_default(),
    }
}

/// Where a parameter sits among its property's parameters of that name.
fn ordinal_of(params: &[IcalParam<'_>], at: usize, name: &str) -> usize {
    params[..at]
        .iter()
        .filter(|held| param_name(held) == name)
        .count()
}

/// The parameter of that name at that position, if the property has one.
fn nth_param<'p, 'a>(
    params: &'p [IcalParam<'a>],
    name: &str,
    at: usize,
) -> Option<&'p IcalParam<'a>> {
    params
        .iter()
        .filter(|held| param_name(held) == name)
        .nth(at)
}

/// Replay one right-side action onto the merged calendar.
fn apply<'a>(
    merged: &mut IcalCst<'a>,
    op: &Op<'a>,
    displaces: Option<&Op<'a>>,
    right: &IcalCst<'a>,
    shift: &Shift<'_>,
) {
    match &op.action {
        IcalMergeAction::ComponentAdded { at } => {
            let Some(source) = find(right, at).cloned() else {
                return;
            };
            let Some(target) = find_mut(merged, &parent(at)) else {
                return;
            };

            let source = IcalItem::Component(alloc::boxed::Box::new(source));

            // NOTE: The addition it beat is replaced where it stood, so the
            // winner neither joins the loser nor renumbers the siblings a
            // position addresses.
            match displaces
                .map(|op| op.path())
                .and_then(|beaten| component_position(target, beaten))
            {
                Some(held) => target.items[held] = source,
                None => target.items.push(source),
            }
        }
        IcalMergeAction::ComponentRemoved { at } => {
            let Some(target) = find_mut(merged, &parent(at)) else {
                return;
            };

            if let Some(held) = component_position(target, at) {
                target.items.remove(held);
            }
        }
        _ => apply_to_line(merged, op, displaces, right, shift),
    }
}

/// Where the component a path names sits among its parent's items.
fn component_position(target: &IcalCst<'_>, at: &IcalComponentPath<'_>) -> Option<usize> {
    let step = at.0.last()?;
    let mut ordinal = 0;

    target.items.iter().position(|item| {
        let IcalItem::Component(child) = item else {
            return false;
        };

        if component_name(child) != step.name {
            return false;
        }

        let held = key(child, ordinal);
        ordinal += 1;
        held == step.key
    })
}

/// Replay a property-level action onto the line it lands on.
fn apply_to_line<'a>(
    merged: &mut IcalCst<'a>,
    op: &Op<'a>,
    displaces: Option<&Op<'a>>,
    right: &IcalCst<'a>,
    shift: &Shift<'_>,
) {
    let action = &op.action;

    let Some(at) = prop_path_of(action) else {
        return;
    };

    // NOTE: The right side's own line is copied, bytes and all, rather than
    // re-encoded from the model, so what lands arrives as it was written. It
    // is addressed by the position it holds in the right side, never by the
    // one its counterpart holds in the base.
    let source = op
        .source
        .as_ref()
        .and_then(|source| {
            find(right, &source.component).and_then(|cst| line_at(cst, source, Some(source.index)))
        })
        .map(terminated);

    let Some(component) = find_mut(merged, &at.component) else {
        return;
    };

    if let IcalMergeAction::PropAdded { .. } = action {
        let Some(source) = source else {
            return;
        };

        // NOTE: The addition it beat is replaced where it stood, so the winner
        // neither joins the loser nor renumbers the lines after it.
        let beaten = displaces.and_then(|op| op.prop()).and_then(|beaten| {
            let ordinal = line_ordinal(component, beaten, Some(beaten.index))?;

            line_position(component, &beaten.name, ordinal)
        });

        match beaten {
            Some(held) => component.items[held] = IcalItem::Prop(source),
            None => component.items.push(IcalItem::Prop(source)),
        }

        return;
    }

    let target = line_ordinal(component, at, shift.translate(at));

    if let IcalMergeAction::PropRemoved { .. } = action {
        if let Some(held) = target.and_then(|ordinal| line_position(component, &at.name, ordinal)) {
            component.items.remove(held);
        }

        return;
    }

    let Some(source) = source else {
        return;
    };

    // NOTE: The line may be gone because the left side removed it while the
    // right side updated it. The update is what survives that stand-off, so the
    // line comes back rather than the update landing nowhere.
    let Some(line) = target.and_then(|ordinal| nth_line_mut(component, &at.name, ordinal)) else {
        component.items.push(IcalItem::Prop(source));
        return;
    };

    match action {
        IcalMergeAction::ValueChanged { .. } => line.value.clone_from(&source.value),
        // NOTE: A list is merged item by item rather than replaced, or the
        // right side's whole value would undo the left side's additions.
        IcalMergeAction::ValueItemAdded { item, .. } => {
            let mut items: Vec<String> = list(line);

            if !items.iter().any(|held| held == item) {
                items.push(item.to_string());
            }

            set_list(line, &items);
        }
        IcalMergeAction::ValueItemRemoved { item, .. } => {
            let kept: Vec<String> = list(line).into_iter().filter(|held| held != item).collect();

            set_list(line, &kept);
        }
        // NOTE: A parameter name may be written more than once on one line
        // (RFC 5545 3.2), so an action addresses the occurrence it named
        // rather than the first of that name.
        IcalMergeAction::ParamRemoved { .. } => {
            if let Slot::Param { name, at } = &op.slot
                && let Some(held) = param_position(line, name, *at)
            {
                line.params.remove(held);
            }
        }
        // NOTE: The parameter is copied off the source line rather than
        // re-encoded from the decoded action: decoding resolves the value
        // escapes and encoding does not put them back, so a re-encoding can
        // write a line break into the head.
        IcalMergeAction::ParamAdded { .. } | IcalMergeAction::ParamChanged { .. } => {
            let Slot::Param { name, at } = &op.slot else {
                return;
            };

            let Some(found) = param_position(&source, name, *at) else {
                return;
            };

            let written = source.params[found].clone();

            match param_position(line, name, *at) {
                Some(held) => line.params[held] = written,
                None => line.params.push(written),
            }
        }
        _ => {}
    }
}

/// Where the parameter of that name at that position sits among a line's
/// parameters.
fn param_position(line: &IcalLine<'_>, name: &str, at: usize) -> Option<usize> {
    line.params
        .iter()
        .enumerate()
        .filter(|(_, held)| held.name.get().to_ascii_uppercase() == name)
        .map(|(held, _)| held)
        .nth(at)
}

/// The items of a line's list value.
fn list(line: &mut IcalLine<'_>) -> Vec<String> {
    IcalValueCursor { line }
        .list()
        .into_iter()
        .map(Cow::into_owned)
        .collect()
}

/// Replace the items of a line's list value.
fn set_list(line: &mut IcalLine<'_>, items: &[String]) {
    IcalValueCursor { line }.set_list(items);
}

/// The property an action lands on, for the actions that land on one.
fn prop_path_of<'p, 'a>(action: &'p IcalMergeAction<'a>) -> Option<&'p IcalPropPath<'a>> {
    match action {
        IcalMergeAction::ComponentAdded { .. } | IcalMergeAction::ComponentRemoved { .. } => None,
        IcalMergeAction::PropAdded { at, .. }
        | IcalMergeAction::PropRemoved { at, .. }
        | IcalMergeAction::ValueChanged { at, .. }
        | IcalMergeAction::ValueItemAdded { at, .. }
        | IcalMergeAction::ValueItemRemoved { at, .. }
        | IcalMergeAction::ParamAdded { at, .. }
        | IcalMergeAction::ParamRemoved { at, .. }
        | IcalMergeAction::ParamChanged { at, .. } => Some(at),
    }
}

/// A path with its last step dropped: the component holding the one it names.
fn parent<'a>(path: &IcalComponentPath<'a>) -> IcalComponentPath<'a> {
    let mut parent = path.clone();
    parent.0.pop();
    parent
}

/// The component a path names.
fn find<'c, 'a>(cst: &'c IcalCst<'a>, path: &IcalComponentPath<'a>) -> Option<&'c IcalCst<'a>> {
    let mut held = cst;

    for step in &path.0 {
        let mut ordinal = 0;

        held = components(held).find(|child| {
            if component_name(child) != step.name {
                return false;
            }

            let matched = key(child, ordinal) == step.key;
            ordinal += 1;
            matched
        })?;
    }

    Some(held)
}

/// The same, mutably.
fn find_mut<'c, 'a>(
    cst: &'c mut IcalCst<'a>,
    path: &IcalComponentPath<'a>,
) -> Option<&'c mut IcalCst<'a>> {
    let mut held = cst;

    for step in &path.0 {
        let mut ordinal = 0;
        held = held.items.iter_mut().find_map(|item| {
            let IcalItem::Component(child) = item else {
                return None;
            };

            if component_name(child) != step.name {
                return None;
            }

            let matched = key(child, ordinal) == step.key;
            ordinal += 1;
            matched.then_some(&mut **child)
        })?;
    }

    Some(held)
}

/// Which of a component's same-named lines a property path names: the one
/// carrying its identity where it has one, and the position it was given
/// otherwise.
fn line_ordinal(
    cst: &IcalCst<'_>,
    at: &IcalPropPath<'_>,
    position: Option<usize>,
) -> Option<usize> {
    let Some(identity) = &at.identity else {
        return position;
    };

    lines(cst)
        .filter(|line| line.name.get().eq_ignore_ascii_case(&at.name))
        .position(|line| value_text(line) == **identity)
}

/// The line a property path names inside a component.
fn line_at<'c, 'a>(
    cst: &'c IcalCst<'a>,
    at: &IcalPropPath<'_>,
    position: Option<usize>,
) -> Option<&'c IcalLine<'a>> {
    let ordinal = line_ordinal(cst, at, position)?;

    lines(cst)
        .filter(|line| line.name.get().eq_ignore_ascii_case(&at.name))
        .nth(ordinal)
}

/// The line of that name at that position inside a component, mutably.
fn nth_line_mut<'c, 'a>(
    cst: &'c mut IcalCst<'a>,
    name: &str,
    at: usize,
) -> Option<&'c mut IcalLine<'a>> {
    cst.items
        .iter_mut()
        .filter_map(|item| match item {
            IcalItem::Prop(line) => Some(line),
            _ => None,
        })
        .filter(|line| !structural(line) && line.name.get().eq_ignore_ascii_case(name))
        .nth(at)
}

/// A copy of a line that is sure to end.
///
/// A side may have been read from a truncated download, and its last line then
/// carries no line ending. Copied into the middle of a calendar it would
/// swallow the line after it, `END:VCALENDAR` included, and the merge would
/// emit bytes its own parser refuses.
fn terminated<'a>(line: &IcalLine<'a>) -> IcalLine<'a> {
    let mut held = line.clone();

    if held.eol.get().is_empty() {
        held.eol = IcalLeaf(Cow::Borrowed("\r\n"));
    }

    held
}

/// Where the line of that name at that position sits among a component's
/// items.
fn line_position(cst: &IcalCst<'_>, name: &str, at: usize) -> Option<usize> {
    let mut ordinal = 0;

    cst.items.iter().position(|item| {
        let IcalItem::Prop(line) = item else {
            return false;
        };

        if structural(line) || !line.name.get().eq_ignore_ascii_case(name) {
            return false;
        }

        let held = ordinal;
        ordinal += 1;
        held == at
    })
}
