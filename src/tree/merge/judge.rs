//! # Judging
//!
//! Whether a right-side action lands in the merged calendar, and what is
//! reported about it.
//!
//! Two actions collide when they occupy the same field. Granularity settles a
//! removal met by an edit rather than the word removal itself: a side dropping
//! one parameter keeps the property, so against a side that removed the
//! property whole it is the one preserving data.
//!
//! A recurrence conflict refuses nothing. Both sides said something true about
//! different parts of one series, and the caller is told only because one may
//! have moved the ground the other stood on.

use alloc::vec::Vec;

use crate::tree::{
    cst::IcalCst,
    line::IcalLine,
    merge::{IcalMerge, IcalMergeAction, IcalMergeReason, IcalPropPath, Op, Slot},
};

/// What a merge decided about one right-side action: whether it lands in the
/// merged calendar, and what to report about it.
pub(super) struct Verdict<'a> {
    pub(super) applies: bool,
    pub(super) reason: Option<IcalMergeReason<'a>>,
}

impl<'a> IcalMerge<'_, 'a> {
    /// Whether a right-side action applies, and what to report about it.
    pub(super) fn judge(
        &self,
        op: &Op<'a>,
        left_ops: &[Op<'a>],
        right_ops: &[Op<'a>],
    ) -> Verdict<'a> {
        if let Some(collision) = left_ops.iter().find(|left| self.collides(left, op)) {
            // NOTE: a removal against an update is not a stand-off: one side
            // says the data is gone and the other says what it now is, and the
            // update survives whichever side it came from.
            return Verdict {
                applies: collision.scraps(op),
                reason: Some(IcalMergeReason::Divergent(collision.action.clone())),
            };
        }

        Verdict {
            // NOTE: The merged calendar is the left side, so an act the left
            // side already performed identically needs no replaying, and
            // replaying an addition would put it there twice.
            applies: !left_ops.iter().any(|held| self.agrees(held, op)),
            // NOTE: A pair the replayed side made in full is one person's own
            // edit seen twice rather than two people disagreeing.
            reason: left_ops
                .iter()
                .find(|left| {
                    left.across_the_series(op)
                        && !right_ops.iter().any(|held| self.agrees(left, held))
                })
                .map(|left| IcalMergeReason::Recurrence(left.action.clone())),
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
            (Slot::Component, Slot::Component) => left.reaches(right) || right.reaches(left),
            (Slot::Component, _) => left.reaches(right),
            (_, Slot::Component) => right.reaches(left),
            _ if !same_prop(left.prop(), right.prop()) => false,
            // NOTE: An addition names a property the base did not hold, and is
            // addressed by a position in the side that wrote it; every other
            // action names one the base held, addressed by its position there.
            // The two numbering systems never name one property.
            _ if left.is_addition() != right.is_addition() => false,
            // NOTE: A property one side removed is a property the other side
            // cannot usefully edit, so a change to its value, to one of its
            // parameters or to one of its list items meets the removal.
            (Slot::Prop, _) | (_, Slot::Prop) => true,
            // NOTE: one of the two values has to go, so a whole-value change
            // on one side meets the other side's item edits rather than
            // letting both land.
            (Slot::Value, Slot::Items) | (Slot::Items, Slot::Value) => true,
            // NOTE: `VALUE` declares what type the value is read as, so
            // retyping it contests every value-level action the other side
            // made: the items it wrote were written for the old type, and
            // keeping both leaves a property whose items contradict its own
            // declared type (RFC 5545 3.8.5.2 for `RDATE`).
            (Slot::Param { name, .. }, Slot::Value | Slot::Items)
            | (Slot::Value | Slot::Items, Slot::Param { name, .. })
                if name == "VALUE" =>
            {
                true
            }
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
    /// An act names where it lands and what it says, never how it is spelt, so
    /// the two sides' own bytes settle it: what one side did is what the other
    /// did only where both wrote the same thing.
    fn agrees(&self, left: &Op<'a>, right: &Op<'a>) -> bool {
        left.action.same_change_as(&right.action) && self.wrote_alike(left, right)
    }

    /// Whether the two sides put the same bytes on the wire for one act.
    ///
    /// What is weighed is what the act wrote: a component or line added, a
    /// value changed, a list item gained, a parameter written. An act that
    /// only takes something away wrote nothing, and what it names lives in the
    /// base both sides share, so the act itself settles it.
    fn wrote_alike(&self, left: &Op<'a>, right: &Op<'a>) -> bool {
        match &right.action {
            IcalMergeAction::ComponentAdded { at } => {
                let held = self.left.at(at).map(IcalCst::to_bytes);

                held.is_some() && held == self.right.at(at).map(IcalCst::to_bytes)
            }
            IcalMergeAction::PropAdded { .. } => {
                let held = self.added_line(self.left, left);

                held.is_some() && held == self.added_line(self.right, right)
            }
            IcalMergeAction::ComponentRemoved { .. }
            | IcalMergeAction::PropRemoved { .. }
            | IcalMergeAction::ValueItemRemoved { .. }
            | IcalMergeAction::ParamRemoved { .. } => true,
            IcalMergeAction::ValueChanged { .. } => {
                let (Some(ours), Some(theirs)) = self.written_lines(left, right) else {
                    return false;
                };

                ours.value.raw_bytes() == theirs.value.raw_bytes()
            }
            IcalMergeAction::ValueItemAdded { item, .. } => {
                let (Some(ours), Some(theirs)) = self.written_lines(left, right) else {
                    return false;
                };

                ours.value.same_item_as(&theirs.value, item)
            }
            IcalMergeAction::ParamAdded { param, .. }
            | IcalMergeAction::ParamChanged { new: param, .. } => {
                let (Some(ours), Some(theirs)) = self.written_lines(left, right) else {
                    return false;
                };

                ours.same_param_bytes_as(&left.slot, theirs, &right.slot, param)
            }
        }
    }

    /// The two lines the two sides wrote their acts on, each in its own side.
    fn written_lines(
        &self,
        left: &Op<'a>,
        right: &Op<'a>,
    ) -> (Option<&IcalLine<'a>>, Option<&IcalLine<'a>>) {
        (
            self.written_line(self.left, left),
            self.written_line(self.right, right),
        )
    }

    /// The line an act was written on, in the side that wrote it.
    pub(super) fn written_line<'c>(
        &self,
        cst: &'c IcalCst<'a>,
        op: &Op<'a>,
    ) -> Option<&'c IcalLine<'a>> {
        let at = op.source.as_ref()?;

        cst.at(&at.component)?.line_at(at, Some(at.index))
    }

    /// The bytes of the line an addition put in one side.
    fn added_line(&self, cst: &IcalCst<'a>, op: &Op<'a>) -> Option<Vec<u8>> {
        let line = self.written_line(cst, op)?;
        let mut out = Vec::new();

        line.write_bytes(&mut out);

        Some(out)
    }
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
