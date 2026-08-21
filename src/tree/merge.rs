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
//! ## What is matched with what
//!
//! A component is matched across the three calendars by its `UID` and its
//! `RECURRENCE-ID`, the identity iCalendar itself uses (RFC 5545 3.8.4.7,
//! 3.8.4.4): an override of one instance is never confused with the series it
//! belongs to, however the two are ordered in the file. A component carrying no
//! `UID` (a `VALARM`, a `STANDARD`, a `VTIMEZONE` observance) is matched by its
//! position among its same-named siblings. Inside a matched component,
//! properties are matched by name, then by equality, then by position;
//! iCalendar has no `PID`, so there is nothing finer to go on.
//!
//! ## What counts as a change
//!
//! A whole property added or removed, a value changed, one item of a list value
//! added or removed, a parameter added, removed or changed. List items merge as
//! a set, both sides' additions and removals applying, so they never collide.
//!
//! ## The three ways a merge can conflict
//!
//! **Divergence.** Both sides changed the same field. The left side's outcome is
//! kept, except where a removal meets an update: there the update wins, because
//! keeping data beats losing it silently.
//!
//! **Recurrence.** One side changed the series (its `RRULE`, `RDATE`, `EXDATE`
//! or start) while the other changed one instance of it. Neither is wrong and
//! both survive, but a rule that moved may have moved the ground the override
//! stood on, so it is reported.
//!
//! **Authority.** An attendee may not rewrite what the organiser owns (RFC 5546
//! 3.2). Set [`right_speaks_for`](IcalMerge::right_speaks_for) to the calendar
//! address the right side edits as, and a right-side change to an
//! organiser-owned property of a component someone else organises is refused and
//! reported. Left unset, no such claim is made and nothing is refused on this
//! ground.

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
        line::IcalLine,
        value::cursor::IcalValueCursor,
    },
    value::IcalValue,
    version::IcalVersion,
};

/// A three-way merge waiting to run.
///
/// The three calendars, plus who the right side speaks for. See the module
/// documentation for the matching, granularity and conflict rules.
pub struct IcalMerge<'m, 'a> {
    /// The common ancestor both sides were derived from.
    pub base: &'m IcalCst<'a>,
    /// One side. Its bytes are the ones the merged calendar keeps.
    pub left: &'m IcalCst<'a>,
    /// The other side. Its changes are replayed onto the left's bytes.
    pub right: &'m IcalCst<'a>,
    /// The calendar address the right side edits on behalf of, when it is an
    /// attendee rather than the organiser of what it changed. Unset means no
    /// claim, and no change is refused for want of authority.
    pub right_speaks_for: Option<Cow<'a, str>>,
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

        for op in &right_ops {
            let verdict = self.judge(op, &left_ops, &base, &left);

            if verdict.applies {
                apply(&mut merged, op, self.right);
            }

            if let Some(reason) = verdict.reason {
                conflicts.push(IcalMergeConflict {
                    right: op.action.clone(),
                    reason,
                });
            }
        }

        IcalMergeReport {
            merged,
            left: left_ops.into_iter().map(|op| op.action).collect(),
            right: right_ops.into_iter().map(|op| op.action).collect(),
            conflicts,
        }
    }

    /// Whether a right-side action applies, and what to report about it.
    fn judge(
        &self,
        op: &Op<'a>,
        left_ops: &[Op<'a>],
        base: &[Node<'_, 'a>],
        left: &[Node<'_, 'a>],
    ) -> Verdict<'a> {
        if let Some(speaker) = &self.right_speaks_for
            && op.organiser_owned
            && organiser_of(op.path(), base, left).is_some_and(|held| held != *speaker)
        {
            return Verdict {
                applies: false,
                reason: Some(IcalMergeReason::Authority),
            };
        }

        if let Some(collision) = left_ops.iter().find(|left| collides(left, op)) {
            // NOTE: A removal against an update is not a stand-off: one side
            // says the data is gone and the other says what it now is. The
            // update survives whichever side it came from, since keeping data
            // beats losing it silently, and the collision is reported either
            // way.
            let applies = collision.action.is_removal() && !op.action.is_removal();

            return Verdict {
                applies,
                reason: Some(IcalMergeReason::Divergent(collision.action.clone())),
            };
        }

        // NOTE: A recurrence conflict refuses nothing. Both sides said
        // something true about different parts of one series, and the caller is
        // told only because one may have moved the ground the other stood on.
        Verdict {
            applies: true,
            reason: left_ops
                .iter()
                .find(|left| across_the_series(left, op))
                .map(|left| IcalMergeReason::Recurrence(left.action.clone())),
        }
    }
}

/// What a merge decided about one right-side action.
struct Verdict<'a> {
    /// Whether the action lands in the merged calendar.
    applies: bool,
    /// What to report about it, if anything.
    reason: Option<IcalMergeReason<'a>>,
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
    /// Both sides changed the same field. The merged calendar holds the left
    /// side's outcome, except where a removal met an update, in which case the
    /// update was kept whichever side it came from.
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

/// One property's address: the component holding it, its name, and which of
/// that component's same-named properties it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalPropPath<'a> {
    /// The component the property belongs to.
    pub component: IcalComponentPath<'a>,
    /// The property name as written.
    pub name: Cow<'a, str>,
    /// The position among the component's properties of that name.
    pub index: usize,
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
    /// One parameter, by name.
    Param(String),
}

/// Whether two actions collide on one field.
fn collides(left: &Op<'_>, right: &Op<'_>) -> bool {
    if left.path() != right.path() {
        return false;
    }

    match (&left.slot, &right.slot) {
        (Slot::Component, Slot::Component) => true,
        // NOTE: A component one side removed is a component the other side
        // cannot usefully edit, so every change inside it collides with the
        // removal rather than quietly applying to something that is gone.
        (Slot::Component, _) | (_, Slot::Component) => true,
        _ if left.prop() != right.prop() => false,
        (Slot::Items, _) | (_, Slot::Items) => false,
        (Slot::Param(left), Slot::Param(right)) => left == right,
        (Slot::Param(_), _) | (_, Slot::Param(_)) => false,
        _ => true,
    }
}

/// Whether one action changed a series and the other one of its instances.
fn across_the_series(left: &Op<'_>, right: &Op<'_>) -> bool {
    let (Some(left), Some(right)) = (left.path().0.last(), right.path().0.last()) else {
        return false;
    };

    let (Some(left_uid), Some(right_uid)) =
        (left.key.split('/').next(), right.key.split('/').next())
    else {
        return false;
    };

    // NOTE: Same UID, and exactly one of the two carries a RECURRENCE-ID: one
    // side is talking about the whole series and the other about one of its
    // occurrences.
    left.name == right.name
        && left_uid == right_uid
        && left.key.contains('/') != right.key.contains('/')
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
fn lines<'c, 'a>(cst: &'c IcalCst<'a>) -> impl Iterator<Item = &'c IcalLine<'a>> {
    cst.items.iter().filter_map(|item| match item {
        IcalItem::Prop(line) => Some(line),
        _ => None,
    })
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
                slot: Slot::Component,
                organiser_owned: whole_component_owned(&node.path),
            });
        }
    }

    for node in base {
        let Some(held) = side.iter().find(|held| held.path == node.path) else {
            continue;
        };

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

        // NOTE: An untouched property pairs with itself before position is
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

        while !base_free.is_empty() && !side_free.is_empty() {
            pairs.push((base_free.remove(0), side_free.remove(0)));
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
                    at,
                },
                slot: Slot::Prop,
            });
        }

        for (b, s) in pairs {
            diff_prop(&base.path, &base_props, b, side_props[s], version, ops);
        }
    }
}

/// The name a line decodes to.
fn decode_name<'a>(line: &IcalLine<'a>) -> IcalPropName<'a> {
    IcalPropName::from(Cow::Owned(line.name.get().to_owned()))
}

/// Where a line sits among its component's same-named properties.
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
    }
}

/// Diff one matched property pair: its parameters, then its value.
fn diff_prop<'a>(
    component: &IcalComponentPath<'a>,
    lines: &[&IcalLine<'a>],
    at: usize,
    side: &IcalLine<'a>,
    version: IcalVersion,
    ops: &mut Vec<Op<'a>>,
) {
    let base = lines[at];
    let at = prop_path(component, lines, at);
    let owned = organiser_owned(component, &decode_name(base));

    let base_prop = base.decode(version);
    let side_prop = side.decode(version);

    for param in &base_prop.params {
        let name = param_name(param);
        let held = side_prop
            .params
            .iter()
            .find(|held| param_name(held) == name);

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
            slot: Slot::Param(name),
            organiser_owned: owned,
        });
    }

    for param in &side_prop.params {
        let name = param_name(param);

        if base_prop.params.iter().any(|held| param_name(held) == name) {
            continue;
        }

        ops.push(Op {
            action: IcalMergeAction::ParamAdded {
                at: at.clone(),
                param: param.clone().into_owned(),
            },
            slot: Slot::Param(name),
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
            list_ops(&at, &old.0, &new.0, owned, ops)
        }
        (IcalValue::DateTimeList(old), IcalValue::DateTimeList(new)) => {
            list_ops(&at, &old.0, &new.0, owned, ops)
        }
        (old, new) => ops.push(Op {
            action: IcalMergeAction::ValueChanged {
                at,
                old: old.clone().into_owned(),
                new: new.clone().into_owned(),
            },
            slot: Slot::Value,
            organiser_owned: owned,
        }),
    }
}

/// The item-by-item difference between two list values.
fn list_ops<'a>(
    at: &IcalPropPath<'a>,
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

/// Replay one right-side action onto the merged calendar.
fn apply<'a>(merged: &mut IcalCst<'a>, op: &Op<'a>, right: &IcalCst<'a>) {
    match &op.action {
        IcalMergeAction::ComponentAdded { at } => {
            let (Some(source), Some(target)) = (find(right, at), find_mut(merged, &parent(at)))
            else {
                return;
            };

            target
                .items
                .push(IcalItem::Component(alloc::boxed::Box::new(source.clone())));
        }
        IcalMergeAction::ComponentRemoved { at } => {
            let (Some(step), Some(target)) = (at.0.last(), find_mut(merged, &parent(at))) else {
                return;
            };

            let step = step.clone();
            let mut ordinal = 0;

            target.items.retain(|item| {
                let IcalItem::Component(child) = item else {
                    return true;
                };

                if component_name(child) != step.name {
                    return true;
                }

                let held = key(child, ordinal);
                ordinal += 1;
                held != step.key
            });
        }
        action => apply_to_line(merged, action, right),
    }
}

/// Replay a property-level action onto the line it lands on.
fn apply_to_line<'a>(merged: &mut IcalCst<'a>, action: &IcalMergeAction<'a>, right: &IcalCst<'a>) {
    let Some(at) = prop_path_of(action) else {
        return;
    };

    let Some(component) = find_mut(merged, &at.component) else {
        return;
    };

    if let IcalMergeAction::PropAdded { .. } = action {
        // NOTE: The right side's own line is copied, bytes and all, rather than
        // re-encoded from the model, so an added property arrives as written.
        if let Some(line) = find(right, &at.component).and_then(|cst| nth_line(cst, at)) {
            component.items.push(IcalItem::Prop(line.clone()));
        }

        return;
    }

    if let IcalMergeAction::PropRemoved { .. } = action {
        let mut index = 0;
        let name = at.name.clone();
        let nth = at.index;

        component.items.retain(|item| {
            let IcalItem::Prop(line) = item else {
                return true;
            };

            if !line.name.get().eq_ignore_ascii_case(&name) {
                return true;
            }

            let held = index;
            index += 1;
            held != nth
        });

        return;
    }

    let Some(source) = find(right, &at.component).and_then(|cst| nth_line(cst, at)) else {
        return;
    };

    // NOTE: The line may be gone because the left side removed it while the
    // right side updated it. The update is what survives that stand-off, so the
    // line comes back rather than the update landing nowhere.
    if nth_line_mut(component, at).is_none() {
        component.items.push(IcalItem::Prop(source.clone()));
        return;
    }

    let Some(line) = nth_line_mut(component, at) else {
        return;
    };

    match action {
        IcalMergeAction::ValueChanged { .. } => line.value = source.value.clone(),
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
        IcalMergeAction::ParamRemoved { param, .. } => {
            let name = param_name(param);
            line.params
                .retain(|held| held.name.get().to_ascii_uppercase() != name);
        }
        IcalMergeAction::ParamAdded { param, .. }
        | IcalMergeAction::ParamChanged { new: param, .. } => {
            let name = param_name(param);
            let encoded = param.encode();

            match line
                .params
                .iter_mut()
                .find(|held| held.name.get().to_ascii_uppercase() == name)
            {
                Some(held) => *held = encoded,
                None => line.params.push(encoded),
            }
        }
        _ => {}
    }
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
        held = components(held)
            .enumerate()
            .find(|(ordinal, child)| {
                component_name(child) == step.name && key(child, *ordinal) == step.key
            })
            .map(|(_, child)| child)?;
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

/// The line a property path names inside a component.
fn nth_line<'c, 'a>(cst: &'c IcalCst<'a>, at: &IcalPropPath<'a>) -> Option<&'c IcalLine<'a>> {
    lines(cst)
        .filter(|line| line.name.get().eq_ignore_ascii_case(&at.name))
        .nth(at.index)
}

/// The same, mutably.
fn nth_line_mut<'c, 'a>(
    cst: &'c mut IcalCst<'a>,
    at: &IcalPropPath<'a>,
) -> Option<&'c mut IcalLine<'a>> {
    cst.items
        .iter_mut()
        .filter_map(|item| match item {
            IcalItem::Prop(line) => Some(line),
            _ => None,
        })
        .filter(|line| line.name.get().eq_ignore_ascii_case(&at.name))
        .nth(at.index)
}
