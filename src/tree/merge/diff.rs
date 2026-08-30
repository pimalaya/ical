//! # Diffing
//!
//! What one side changed relative to the base, as one [`Op`] per observed
//! change.
//!
//! Inside a matched component, a property is matched down one ladder: a
//! synchronisation identity, which iCalendar does not define for a property so
//! the rung is empty here, then a natural identity, then equality, then
//! position.
//!
//! A property that may occur more than once and whose value names a thing
//! outside the calendar is identified by that value: `ATTENDEE` by its
//! calendar user address, `ATTACH` by its URI or inline binary, `RELATED-TO`
//! by the `UID` it points at, `CONFERENCE` and `IMAGE` by their URI.
//!
//! A different calendar address is a different person, so two properties
//! carrying different identities are never matched with each other, and a
//! value two siblings share tells neither of them apart, so both of those fall
//! back to their positions.
//!
//! An identity is compared lowercased and written back exactly, so an address
//! meets the other case it was written in while the line keeps its own bytes.
//!
//! ## What counts as a change
//!
//! A whole property added or removed, a value changed, one item of a list
//! value added or removed, a parameter added, removed or changed. List items
//! merge as a set, both sides' additions and removals applying, so they never
//! collide.

use alloc::{
    borrow::{Cow, ToOwned},
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::{
    param::IcalParam,
    prop::{IcalPropKind, IcalPropName},
    tree::{
        line::IcalLine,
        merge::{IcalComponentPath, IcalMergeAction, IcalPropPath, Op, Slot, node::Node},
    },
    value::IcalValue,
    version::IcalVersion,
};

/// One side against the base, as the calendars they walk to.
pub(super) struct Diff<'n, 'c, 'a> {
    pub(super) base: &'n [Node<'c, 'a>],
    pub(super) side: &'n [Node<'c, 'a>],
    pub(super) version: IcalVersion,
}

impl<'a> Diff<'_, '_, 'a> {
    /// Every change the side made relative to the base.
    pub(super) fn run(&self) -> Vec<Op<'a>> {
        let mut ops = Vec::new();

        for node in self.base {
            if !self.side.iter().any(|held| held.path == node.path)
                && !self.removed_above(&node.path)
            {
                ops.push(Op {
                    action: IcalMergeAction::ComponentRemoved {
                        at: node.path.clone(),
                    },
                    source: None,
                    slot: Slot::Component,
                });
            }
        }

        for node in self.side {
            if !self.base.iter().any(|held| held.path == node.path) && !self.added_above(&node.path)
            {
                ops.push(Op {
                    action: IcalMergeAction::ComponentAdded {
                        at: node.path.clone(),
                    },
                    source: None,
                    slot: Slot::Component,
                });
            }
        }

        // NOTE: A calendar may hold two components at one path, a `UID`
        // written twice with no `RECURRENCE-ID` telling them apart, so each
        // side component is matched once: matching both base components
        // against the same one would report the difference between them as a
        // change either side made.
        let mut taken = vec![false; self.side.len()];

        for node in self.base {
            let Some((at, held)) = self
                .side
                .iter()
                .enumerate()
                .find(|(at, held)| !taken[*at] && held.path == node.path)
            else {
                continue;
            };

            taken[at] = true;

            self.component(node, held, &mut ops);
        }

        ops
    }

    /// Whether an ancestor of this path is itself missing from the side, so
    /// the removal is already reported one level up.
    fn removed_above(&self, path: &IcalComponentPath<'_>) -> bool {
        path.ancestors().any(|above| {
            self.base.iter().any(|node| node.path == above)
                && !self.side.iter().any(|node| node.path == above)
        })
    }

    /// The mirror of [`Diff::removed_above`] for an addition.
    fn added_above(&self, path: &IcalComponentPath<'_>) -> bool {
        path.ancestors().any(|above| {
            self.side.iter().any(|node| node.path == above)
                && !self.base.iter().any(|node| node.path == above)
        })
    }

    /// Diff the properties of one matched component pair.
    fn component(&self, base: &Node<'_, 'a>, side: &Node<'_, 'a>, ops: &mut Vec<Op<'a>>) {
        let base_props: Vec<&IcalLine<'a>> = base.cst.prop_lines().collect();
        let side_props: Vec<&IcalLine<'a>> = side.cst.prop_lines().collect();

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
            let mut pairs = Vec::new();

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

            // NOTE: An untouched property pairs with itself before position is
            // consulted, so adding one line does not renumber every line after
            // it.
            let mut b = 0;
            while b < base_free.len() {
                let same = side_free.iter().position(|&s| {
                    base_props[base_free[b]].decode(self.version)
                        == side_props[s].decode(self.version)
                });

                match same {
                    Some(s) => pairs.push((base_free.remove(b), side_free.remove(s))),
                    None => b += 1,
                }
            }

            // NOTE: Position only tells apart properties iCalendar gives no
            // identity of their own. A calendar address that matched nothing
            // names a person who left, never a person the other side renamed.
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
                let at = IcalPropPath::of(&base.path, &base_props, index);

                ops.push(Op {
                    action: IcalMergeAction::PropRemoved {
                        value: line.decode(self.version).value.into_owned(),
                        at,
                    },
                    source: None,
                    slot: Slot::Prop,
                });
            }

            for index in side_free {
                let line = side_props[index];
                let at = IcalPropPath::of(&side.path, &side_props, index);

                ops.push(Op {
                    action: IcalMergeAction::PropAdded {
                        value: line.decode(self.version).value.into_owned(),
                        at: at.clone(),
                    },
                    source: Some(at),
                    slot: Slot::Prop,
                });
            }

            for (b, s) in pairs {
                self.prop(&base.path, &base_props, b, &side_props, s, ops);
            }
        }
    }

    /// Diff one matched property pair: its parameters, then its value.
    fn prop(
        &self,
        component: &IcalComponentPath<'a>,
        lines: &[&IcalLine<'a>],
        at: usize,
        side_lines: &[&IcalLine<'a>],
        side_at: usize,
        ops: &mut Vec<Op<'a>>,
    ) {
        let base = lines[at];
        let side = side_lines[side_at];
        let at = IcalPropPath::of(component, lines, at);
        let source = IcalPropPath::of(component, side_lines, side_at);

        let base_prop = base.decode(self.version);
        let side_prop = side.decode(self.version);

        for (index, param) in base_prop.params.iter().enumerate() {
            let name = param.merge_name();
            let ordinal = ordinal_of(&base_prop.params, index, &name);
            let held = nth_param(&side_prop.params, &name, ordinal);

            // NOTE: the raw nodes are what is compared, not the decoded
            // parameters: a single-valued parameter decodes its first value
            // alone, so two parameters differing past the first `,` decode
            // alike and the edit is never seen.
            let action = match held {
                None => IcalMergeAction::ParamRemoved {
                    at: at.clone(),
                    param: param.clone().into_owned(),
                },
                Some(held) if !base.params[index].same_param_as(&side.params[held]) => {
                    IcalMergeAction::ParamChanged {
                        at: at.clone(),
                        old: param.clone().into_owned(),
                        new: side_prop.params[held].clone().into_owned(),
                    }
                }
                Some(_) => continue,
            };

            ops.push(Op {
                action,
                source: Some(source.clone()),
                slot: Slot::Param { name, at: ordinal },
            });
        }

        for (index, param) in side_prop.params.iter().enumerate() {
            let name = param.merge_name();
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
            });
        }

        if base.value.same_value_as(&side.value) {
            return;
        }

        match (&base_prop.value, &side_prop.value) {
            // NOTE: A list is a set: both sides' additions and both sides'
            // removals apply, so two sides editing one list never collide.
            (IcalValue::TextList(old), IcalValue::TextList(new)) => {
                list_ops(&at, &source, &old.0, &new.0, ops)
            }
            (IcalValue::DateTimeList(old), IcalValue::DateTimeList(new)) => {
                list_ops(&at, &source, &old.0, &new.0, ops)
            }
            (old, new) => ops.push(Op {
                action: IcalMergeAction::ValueChanged {
                    at,
                    old: old.clone().into_owned(),
                    new: new.clone().into_owned(),
                },
                source: Some(source),
                slot: Slot::Value,
            }),
        }
    }
}

impl<'a> IcalLine<'a> {
    /// The identity a property name carries, read off one line.
    fn prop_identity(&self) -> Option<Cow<'a, str>> {
        let name = IcalPropName::from(Cow::Owned(self.name.get().to_owned()));
        let identified = matches!(
            name,
            IcalPropName::Kind(
                IcalPropKind::Attendee
                    | IcalPropKind::Attach
                    | IcalPropKind::RelatedTo
                    | IcalPropKind::Conference
                    | IcalPropKind::Image
            )
        );

        identified.then(|| Cow::Owned(self.value_key()))
    }
}

/// The value that tells a property from its same-named siblings.
///
/// A property that may occur more than once and whose value names a thing
/// outside the calendar is that thing: an `ATTENDEE` a calendar user address
/// (RFC 5545 3.8.4.1), an `ATTACH` a URI or inline binary (3.8.1.1), a
/// `RELATED-TO` a `UID` (3.8.4.5), a `CONFERENCE` or `IMAGE` a URI (RFC 7986).
///
/// Every other property has none: either it may occur only once, so its name
/// already tells it apart, or its value is the datum being edited, and keying
/// on it would make every edit a replacement.
///
/// An identity that does not tell a property from its siblings is no identity:
/// a value written twice in one component names both, so those fall back to
/// their positions, and a sibling still alone with its value keeps its own.
pub(super) fn identity_in<'a>(lines: &[&IcalLine<'a>], at: usize) -> Option<Cow<'a, str>> {
    let held = lines[at].prop_identity()?;
    let name = lines[at].name.get();

    let twice = lines.iter().enumerate().any(|(index, line)| {
        index != at
            && line.name.get().eq_ignore_ascii_case(name)
            && line.prop_identity().is_some_and(|line| line == held)
    });

    (!twice).then_some(held)
}

/// The item-by-item difference between two list values.
fn list_ops<'a>(
    at: &IcalPropPath<'a>,
    source: &IcalPropPath<'a>,
    old: &[Cow<'_, str>],
    new: &[Cow<'_, str>],
    ops: &mut Vec<Op<'a>>,
) {
    let (added, removed) = list_diff(old, new);

    for item in removed {
        ops.push(Op {
            action: IcalMergeAction::ValueItemRemoved {
                at: at.clone(),
                item: Cow::Owned(item.to_string()),
            },
            source: Some(source.clone()),
            slot: Slot::Items,
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
        });
    }
}

/// Diff two value lists as multisets, so a repeated item is matched one for
/// one rather than by mere membership.
///
/// Set membership loses a duplicate: dropping one `a` from `a,a,b` leaves an
/// `a` in the new list, and a set diff then reports no removal at all.
fn list_diff<'a>(
    old: &[Cow<'a, str>],
    new: &[Cow<'a, str>],
) -> (Vec<Cow<'a, str>>, Vec<Cow<'a, str>>) {
    let mut removed: Vec<Option<&Cow<'a, str>>> = old.iter().map(Some).collect();
    let mut added = Vec::new();

    for item in new {
        let kept = removed
            .iter()
            .position(|old| old.is_some_and(|old| old.as_ref() == item.as_ref()));

        match kept {
            Some(i) => removed[i] = None,
            None => added.push(item.clone()),
        }
    }

    let removed = removed.into_iter().flatten().cloned().collect();

    (added, removed)
}

/// Where a parameter sits among its property's parameters of that name.
fn ordinal_of(params: &[IcalParam<'_>], at: usize, name: &str) -> usize {
    params[..at]
        .iter()
        .filter(|held| held.merge_name() == name)
        .count()
}

/// The position of the parameter of that name at that ordinal, if the property
/// has one. The index addresses the decoded list and its raw parameter nodes
/// alike, which a decode maps one for one.
fn nth_param(params: &[IcalParam<'_>], name: &str, at: usize) -> Option<usize> {
    params
        .iter()
        .enumerate()
        .filter(|(_, held)| held.merge_name() == name)
        .map(|(index, _)| index)
        .nth(at)
}
