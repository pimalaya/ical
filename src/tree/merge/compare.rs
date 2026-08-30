//! # Agreement
//!
//! What counts as two sides having performed one act.
//!
//! Two sides agree only where they wrote the same bytes. A decode is not
//! injective, so `\N` and `\n` read alike (RFC 5545 3.3.11) while saying
//! different things on the wire, and reading two such lines as one act would
//! drop the difference without a word.
//!
//! The right side's act is instead judged normally, meets the left side's, and
//! is reported. An act that only takes something away wrote no bytes, and what
//! it names lives in the base both sides share, so the act itself settles it.
//!
//! The one exception is a parameter the specification gives no order:
//! `DELEGATED-FROM` and `DELEGATED-TO` (RFC 5545 3.2.4, 3.2.5), `MEMBER`
//! (3.2.11) and `FEATURE` (RFC 7986 6.3). Those hold lists rather than
//! sequences, so two sides writing one list in two orders wrote one parameter
//! and compare as a set.

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    param::IcalParam,
    tree::{
        codec::unescape::unescape_param,
        line::IcalLine,
        merge::{IcalMergeAction, Slot},
        param::node::IcalParamNode,
        value::node::IcalValueNode,
    },
};

impl IcalValueNode<'_> {
    /// Whether two raw value nodes say the same thing, component by
    /// component.
    ///
    /// The comparison is on the nodes rather than the decoded values: a
    /// decoded value reads its own kind's shape, and a text value reads one
    /// component alone, so two lines differing past the first `;` decode
    /// alike.
    pub(super) fn same_value_as(&self, other: &IcalValueNode<'_>) -> bool {
        // NOTE: two calendars of different versions escape values by different
        // rules, so they share no decoding to compare through. Only identical
        // bytes are then certainly the same value.
        if self.escaper != other.escaper {
            return self.raw_bytes() == other.raw_bytes();
        }

        let count = self.component_count().max(other.component_count());

        (0..count).all(|i| {
            component_eq(
                &self.decode_component_list(i),
                &other.decode_component_list(i),
            )
        })
    }

    /// Whether two sides spelled one item of a list value the same way on the
    /// wire.
    pub(super) fn same_item_as(&self, other: &IcalValueNode<'_>, item: &str) -> bool {
        let raw = |node: &IcalValueNode<'_>| -> Option<Vec<u8>> {
            let at = node
                .decode_list()
                .iter()
                .position(|held| held.as_ref() == item)?;

            node.raw_list().into_iter().nth(at)
        };

        match (raw(self), raw(other)) {
            (Some(ours), Some(theirs)) => ours == theirs,
            _ => false,
        }
    }

    /// The serialized bytes of the node, for comparing across escaping modes.
    pub(super) fn raw_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write_bytes(&mut out);
        out
    }
}

impl IcalParamNode<'_> {
    /// Whether two raw parameter nodes say the same thing, value by value.
    ///
    /// On the nodes rather than the decoded parameters, for the reason
    /// [`IcalValueNode::same_value_as`] gives: a single-valued parameter reads
    /// its first value alone, so two differing past the first `,` decode
    /// alike.
    pub(super) fn same_param_as(&self, other: &IcalParamNode<'_>) -> bool {
        // NOTE: two calendars of different versions encode parameters by
        // different rules, so they share no decoding to compare through. Only
        // identical bytes are then certainly the same parameter.
        if self.escaper != other.escaper {
            return self.raw_bytes() == other.raw_bytes();
        }

        self.values.len() == other.values.len()
            && self.values.iter().zip(&other.values).all(|(ours, theirs)| {
                unescape_param(ours.get(), self.escaper)
                    == unescape_param(theirs.get(), other.escaper)
            })
    }

    /// The serialized bytes of the node, for comparing across escaping modes.
    pub(super) fn raw_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write_bytes(&mut out);
        out
    }

    /// The node's values in a stable order, for comparing them as a set.
    fn sorted_values(&self) -> Vec<String> {
        let mut values: Vec<String> = self
            .values
            .iter()
            .map(|leaf| leaf.get().to_string())
            .collect();
        values.sort_unstable();
        values
    }
}

impl IcalParam<'_> {
    /// The parameter's name, the key two sides' parameters are matched on.
    pub(super) fn merge_name(&self) -> String {
        match self {
            IcalParam::Unknown { name, .. } => name.to_ascii_uppercase(),
            known => known
                .kind()
                .map(|kind| kind.to_ascii_uppercase())
                .unwrap_or_default(),
        }
    }

    /// Whether the parameter's values are a set rather than a sequence.
    ///
    /// Two sides writing them in two orders then wrote one parameter.
    /// `DELEGATED-FROM` and `DELEGATED-TO` (RFC 5545 3.2.4, 3.2.5), `MEMBER`
    /// (3.2.11) and `FEATURE` (RFC 7986 6.3) each hold a list the
    /// specification gives no order, so no arrangement means more than
    /// another.
    pub(super) fn is_unordered(&self) -> bool {
        matches!(
            self,
            IcalParam::DelegatedFrom(_)
                | IcalParam::DelegatedTo(_)
                | IcalParam::Member(_)
                | IcalParam::Feature(_)
        )
    }

    /// Whether two parameters carry the same value, a list parameter the
    /// specification gives no order compared as a set.
    pub(super) fn same_value_as(&self, other: &IcalParam<'_>) -> bool {
        match (self, other) {
            (IcalParam::DelegatedFrom(ours), IcalParam::DelegatedFrom(theirs))
            | (IcalParam::DelegatedTo(ours), IcalParam::DelegatedTo(theirs))
            | (IcalParam::Member(ours), IcalParam::Member(theirs))
            | (IcalParam::Feature(ours), IcalParam::Feature(theirs)) => {
                sorted(ours) == sorted(theirs)
            }
            (ours, theirs) => ours == theirs,
        }
    }
}

impl IcalLine<'_> {
    /// Where the parameter of that name at that position sits among the line's
    /// parameters.
    pub(super) fn param_position(&self, name: &str, at: usize) -> Option<usize> {
        self.params
            .iter()
            .enumerate()
            .filter(|(_, held)| held.name.get().to_ascii_uppercase() == name)
            .map(|(held, _)| held)
            .nth(at)
    }

    /// Whether two sides spelled one parameter the same way on the wire.
    ///
    /// A parameter the specification gives no order compares as a set of raw
    /// values, for the reason [`IcalParam::is_unordered`] gives; every other
    /// parameter compares whole, so how it is written is part of what it says.
    pub(super) fn same_param_bytes_as(
        &self,
        our_slot: &Slot,
        theirs: &IcalLine<'_>,
        their_slot: &Slot,
        param: &IcalParam<'_>,
    ) -> bool {
        let (
            Slot::Param {
                name: our_name,
                at: our_at,
            },
            Slot::Param {
                name: their_name,
                at: their_at,
            },
        ) = (our_slot, their_slot)
        else {
            return false;
        };

        let (Some(ours), Some(theirs)) = (
            self.param_position(our_name, *our_at)
                .map(|held| &self.params[held]),
            theirs
                .param_position(their_name, *their_at)
                .map(|held| &theirs.params[held]),
        ) else {
            return false;
        };

        if !param.is_unordered() {
            return ours.raw_bytes() == theirs.raw_bytes();
        }

        ours.name.get().eq_ignore_ascii_case(theirs.name.get())
            && ours.sorted_values() == theirs.sorted_values()
    }
}

impl IcalMergeAction<'_> {
    /// Whether two actions are the same change, before the bytes each side
    /// wrote are weighed.
    ///
    /// Equality is exact but for a parameter the specification gives no order,
    /// whose values compare as a set: see [`IcalParam::same_value_as`].
    pub(super) fn same_change_as(&self, other: &IcalMergeAction<'_>) -> bool {
        use IcalMergeAction::{ParamAdded, ParamChanged, ParamRemoved};

        match (self, other) {
            (
                ParamAdded {
                    at: our_at,
                    param: ours,
                },
                ParamAdded {
                    at: their_at,
                    param: theirs,
                },
            )
            | (
                ParamRemoved {
                    at: our_at,
                    param: ours,
                },
                ParamRemoved {
                    at: their_at,
                    param: theirs,
                },
            ) => our_at == their_at && ours.same_value_as(theirs),
            (
                ParamChanged {
                    at: our_at,
                    old: our_old,
                    new: our_new,
                },
                ParamChanged {
                    at: their_at,
                    old: their_old,
                    new: their_new,
                },
            ) => {
                our_at == their_at
                    && our_old.same_value_as(their_old)
                    && our_new.same_value_as(their_new)
            }
            (ours, theirs) => ours == theirs,
        }
    }
}

/// Whether one component's values match, an all-empty component counting as
/// equal to another all-empty one however many pieces each was written with.
fn component_eq(old: &[Cow<'_, str>], new: &[Cow<'_, str>]) -> bool {
    let eq = old.len() == new.len()
        && old
            .iter()
            .zip(new)
            .all(|(old, new)| old.as_ref() == new.as_ref());

    eq || (old.iter().all(|value| value.is_empty()) && new.iter().all(|value| value.is_empty()))
}

/// A list parameter's values in a stable order, for comparing them as a set.
fn sorted<'v>(values: &'v [Cow<'_, str>]) -> Vec<&'v str> {
    let mut items: Vec<&str> = values.iter().map(Cow::as_ref).collect();
    items.sort_unstable();
    items
}
