//! # Three-way merge
//!
//! Reconcile two divergent edits of a calendar against their common base.
//!
//! [`IcalMerge::merge`] is the unit a synchronisation engine needs: given a
//! base calendar and two calendars derived from it, it reports what each side
//! changed relative to the base and builds one merged calendar.
//!
//! Never last-writer-wins: a field only one side touched is taken from that
//! side, and a field both sides touched is a conflict, reported so a caller
//! can resolve it differently.
//!
//! The merged calendar starts as a clone of the left one, so the left side's
//! bytes are there exactly as they were, folds included; the right side's
//! actions are then replayed line by line, so every line the right side did
//! not touch keeps its bytes too.
//!
//! ## Ours and theirs
//!
//! The left side is git's `ours` and the right side is git's `theirs`. The
//! left side supplies the baseline, so its folding, its parameter casing and
//! its property order come out untouched, and it keeps its own value where
//! both sides wrote one into a single field.
//!
//! One side answers both questions on purpose. A caller reaches for a merge
//! holding a version it is merging into, and that version is the one it would
//! rather not churn and the one it means to keep.
//!
//! Every collision is reported either way, so a caller wanting the other value
//! puts it to somebody rather than asking the merge to guess.
//!
//! ## The four steps
//!
//! `node` addresses a calendar: it walks it into components, each carrying the
//! `UID` and `RECURRENCE-ID` path that names it, and finds again in one
//! calendar what was read in another.
//!
//! `diff` matches a side against the base and reports one change per field,
//! down to a single list item or parameter. `compare` says when two sides
//! performed one act, which is only where they wrote the same bytes.
//!
//! `judge` decides whether the right side's act lands and what to report about
//! it, and `replay` puts the ones that land onto the left side's bytes.
//!
//! ## The two ways a merge can conflict
//!
//! **Divergence.** Both sides changed the same field. The left side's outcome
//! is kept, except where a removal meets an update: there the update wins
//! whichever side it came from, because keeping data beats losing it silently.
//!
//! **Recurrence.** One side changed what defines the series (its `DTSTART`,
//! `DTEND`, `DURATION`, `RRULE`, `RDATE` or `EXDATE`, or the series component
//! itself) while the other changed one instance of it.
//!
//! Neither is wrong and both survive, but a rule that moved may have moved the
//! ground the override stood on, so it is reported. A change to anything else
//! the series carries cannot have moved an occurrence and is not reported
//! against one.

mod compare;
mod diff;
mod judge;
mod node;
mod op;
mod replay;

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    param::IcalParam,
    tree::{
        cst::IcalCst,
        merge::{
            diff::Diff,
            op::{Op, Slot},
            replay::Shift,
        },
    },
    value::IcalValue,
};

/// A three-way merge waiting to run.
///
/// See the module documentation for the matching, granularity and conflict
/// rules.
pub struct IcalMerge<'m, 'a> {
    /// The common ancestor both sides were derived from.
    pub base: &'m IcalCst<'a>,
    /// The side being merged into, git's `ours`. The merged calendar is built
    /// from its bytes, and a collision neither side settles keeps its value.
    pub left: &'m IcalCst<'a>,
    /// The side being merged in, git's `theirs`. Its changes are replayed onto
    /// the left's bytes.
    pub right: &'m IcalCst<'a>,
}

impl<'a> IcalMerge<'_, 'a> {
    /// Run the merge.
    pub fn merge(self) -> IcalMergeReport<'a> {
        let version = self.base.version();

        let base = self.base.nodes();
        let left = self.left.nodes();
        let right = self.right.nodes();

        let left_ops = Diff {
            base: &base,
            side: &left,
            version,
        }
        .run();
        let right_ops = Diff {
            base: &base,
            side: &right,
            version,
        }
        .run();

        let mut merged = self.left.clone();
        let mut conflicts = Vec::new();
        let mut applicable = Vec::new();

        for op in &right_ops {
            let verdict = self.judge(op, &left_ops, &right_ops);

            if verdict.applies {
                applicable.push(op);
            }

            if let Some(reason) = verdict.reason {
                conflicts.push(IcalMergeConflict {
                    right: op.action.clone(),
                    reason,
                });
            }
        }

        applicable.sort_by_key(|op| op.replay_order());

        let shift = Shift::of(&left_ops);
        let mut restored = Vec::new();

        for op in applicable {
            self.apply(&mut merged, op, &shift, &mut restored);
        }

        IcalMergeReport {
            merged,
            left: left_ops.into_iter().map(|op| op.action).collect(),
            right: right_ops.into_iter().map(|op| op.action).collect(),
            conflicts,
        }
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
    /// Both sides changed the same field. The merged calendar holds the left
    /// side's outcome, except where a removal met an update, in which case the
    /// update was kept whichever side it came from. The action carried here is
    /// the left side's, beside the right side's on the conflict itself.
    Divergent(IcalMergeAction<'a>),
    /// One side changed a series and the other changed one of its instances.
    /// Both survive in the merged calendar; a rule that moved may have moved
    /// the ground the override stood on, which is why this is said out loud.
    Recurrence(IcalMergeAction<'a>),
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
    /// The value that tells the property from its same-named siblings.
    ///
    /// Where iCalendar gives it one: the calendar user address of an
    /// `ATTENDEE`, the URI or inline binary of an `ATTACH`, the `UID` a
    /// `RELATED-TO` points at, the URI of a `CONFERENCE` or an `IMAGE`.
    /// Lowercased, since matching normalises and writing is exact.
    ///
    /// `None` for every other property, whose position then tells it from its
    /// siblings, and `None` too for a value a same-named sibling repeats,
    /// which tells neither of them apart.
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

impl<'a> IcalMergeAction<'a> {
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

    /// The property the action lands on, for the actions that land on one.
    fn prop_path(&self) -> Option<&IcalPropPath<'a>> {
        match self {
            Self::ComponentAdded { .. } | Self::ComponentRemoved { .. } => None,
            Self::PropAdded { at, .. }
            | Self::PropRemoved { at, .. }
            | Self::ValueChanged { at, .. }
            | Self::ValueItemAdded { at, .. }
            | Self::ValueItemRemoved { at, .. }
            | Self::ParamAdded { at, .. }
            | Self::ParamRemoved { at, .. }
            | Self::ParamChanged { at, .. } => Some(at),
        }
    }
}
