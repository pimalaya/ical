//! # Actions in flight
//!
//! An [`Op`] is one diffed change with what the merge needs to route and judge
//! it: the field it occupies, and where the side that wrote it holds the line
//! the replay reads its bytes from.
//!
//! The [`Slot`] is what two sides collide on. A component reaches everything
//! nested in it, a property reaches its own value, items and parameters, and a
//! parameter reaches only the occurrence it named.

use alloc::{borrow::Cow, string::String};

use core::cmp::Reverse;

use crate::{
    prop::{IcalPropKind, IcalPropName},
    tree::merge::{IcalComponentPath, IcalMergeAction, IcalPropPath},
};

/// One diffed change, with what the merge needs to route and judge it: the
/// field it occupies, and where the side that wrote it holds the line the
/// replay reads its bytes from, which a removal and a whole component lack.
pub(super) struct Op<'a> {
    pub(super) action: IcalMergeAction<'a>,
    pub(super) source: Option<IcalPropPath<'a>>,
    pub(super) slot: Slot,
}

/// The field of a property an action occupies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Slot {
    /// The whole component.
    Component,
    /// The whole property.
    Prop,
    /// The whole value.
    Value,
    /// The items of a list value, which merge as a set and never collide.
    Items,
    /// One parameter, by uppercase name and by its position among the
    /// property's parameters of that name.
    Param { name: String, at: usize },
}

impl<'a> Op<'a> {
    /// The component the action lands in.
    pub(super) fn path(&self) -> &IcalComponentPath<'a> {
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
    pub(super) fn prop(&self) -> Option<&IcalPropPath<'a>> {
        self.action.prop_path()
    }

    /// Whether the action puts a property the base did not hold.
    pub(super) fn is_addition(&self) -> bool {
        matches!(self.action, IcalMergeAction::PropAdded { .. })
    }

    /// Whether a component-level action takes away or replaces another's
    /// target.
    ///
    /// A component one side removed or added is a component the other side
    /// cannot usefully edit, at any depth. Two removals overlapping are left
    /// alone: both sides agreed the data goes, and saying so would be noise.
    pub(super) fn reaches(&self, below: &Op<'_>) -> bool {
        below.path().0.starts_with(&self.path().0)
            && (self.path() == below.path() || !below.action.is_removal())
    }

    /// Whether this action takes away what the other one still works on.
    ///
    /// Granularity settles it rather than the word removal: a side dropping
    /// one parameter keeps the property, so against a side that removed the
    /// property whole it is the one preserving data. Two actions at one
    /// granularity are a stand-off unless exactly one removes.
    pub(super) fn scraps(&self, other: &Op<'_>) -> bool {
        if !self.action.is_removal() {
            return false;
        }

        match (&self.slot, &other.slot) {
            (Slot::Component, Slot::Component) | (Slot::Prop, Slot::Prop) => {
                !other.action.is_removal()
            }
            (Slot::Component, _) | (Slot::Prop, _) => true,
            _ => !other.action.is_removal(),
        }
    }

    /// Whether this action changed what defines a series and the other one of
    /// its instances.
    pub(super) fn across_the_series(&self, other: &Op<'_>) -> bool {
        let (Some(one), Some(two)) = (self.path().0.last(), other.path().0.last()) else {
            return false;
        };

        let (Some(our_uid), Some(their_uid)) =
            (one.key.split('/').next(), two.key.split('/').next())
        else {
            return false;
        };

        // NOTE: Same UID, and exactly one of the two carries a RECURRENCE-ID:
        // one side is talking about the whole series and the other about one
        // of its occurrences.
        if one.name != two.name
            || our_uid != their_uid
            || one.key.contains('/') == two.key.contains('/')
        {
            return false;
        }

        let series = if one.key.contains('/') { other } else { self };

        series.defines_the_set()
    }

    /// Whether the action changed what a recurrence set is made of, rather
    /// than something the series merely describes (RFC 5545 3.8.5).
    fn defines_the_set(&self) -> bool {
        let Some(at) = self.prop() else {
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

    /// Where the action sits in the order the replay applies them.
    ///
    /// A component and a property carrying no identity of its own are
    /// addressed by the position they held in the base, and taking one out
    /// renumbers every same-named one after it. Removals therefore go last,
    /// highest position first, so each still names in the merged calendar what
    /// it named in the base.
    ///
    /// Everything else keeps the order the diff produced, which a stable sort
    /// preserves.
    pub(super) fn replay_order(&self) -> (u8, Reverse<usize>) {
        let last = match &self.action {
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
}
