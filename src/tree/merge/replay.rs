//! # Replay
//!
//! Putting the right side's applicable actions onto the left side's bytes.
//!
//! The right side's own line is copied, bytes and all, rather than re-encoded
//! from the model, so what lands arrives as it was written. It is addressed by
//! the position it holds in the right side, never by the one its counterpart
//! holds in the base.
//!
//! A position an action carries is the one its target held in the base, and
//! the merged calendar starts as the left side's own tree, so the position
//! moves twice on the way in: up past everything that side removed below, then
//! down past everything it added at or before where it lands. That is what
//! [`Shift`] holds.
//!
//! An addition is the exception, since it names a property the base did not
//! hold: it carries the position it holds in the side that added it, and never
//! meets an action addressed in the base.
//!
//! What the baseline side took away out from under an act that writes
//! something comes back rather than the act landing nowhere, at both
//! granularities: the line alone where a property was removed, the whole
//! component where the component holding it was. That is what [`Restored`]
//! holds.

use core::iter;

use alloc::{
    borrow::Cow,
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use crate::tree::{
    cst::{IcalCst, IcalItem},
    line::IcalLine,
    merge::{IcalComponentPath, IcalMerge, IcalMergeAction, IcalPropPath, Op, Slot},
    value::cursor::IcalValueCursor,
};

/// How many members the baseline side took out of each group of same-named
/// properties, so a position measured in the base still names its own target
/// in the merged calendar.
pub(super) struct Shift<'a> {
    removed: Vec<(&'a IcalComponentPath<'a>, String, usize)>,
    added: Vec<(&'a IcalComponentPath<'a>, String, usize)>,
}

impl<'a> Shift<'a> {
    /// Read it off the baseline side's own removals and additions.
    pub(super) fn of(ops: &'a [Op<'a>]) -> Self {
        let mut removed = Vec::new();
        let mut added = Vec::new();

        for op in ops {
            match &op.action {
                IcalMergeAction::PropRemoved { at, .. } => {
                    removed.push((&at.component, at.name.to_ascii_uppercase(), at.index));
                }
                IcalMergeAction::PropAdded { at, .. } => {
                    added.push((&at.component, at.name.to_ascii_uppercase(), at.index));
                }
                _ => {}
            }
        }

        added.sort_by_key(|(_, _, index)| *index);

        Self { removed, added }
    }

    /// Where a base position sits in the merged calendar.
    ///
    /// `None` where the baseline side removed the very property the position
    /// names.
    ///
    /// Reading the removals alone made a left-side insertion address the line
    /// before the one meant, which edited a property nobody had touched.
    fn translate(&self, at: &IcalPropPath<'_>) -> Option<usize> {
        let name = at.name.to_ascii_uppercase();
        let mut shift = 0;

        for (component, held, index) in &self.removed {
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

        let mut position = at.index - shift;

        for (component, held, index) in &self.added {
            if **component != at.component || *held != name {
                continue;
            }

            if *index <= position {
                position += 1;
            }
        }

        Some(position)
    }
}

/// What the replay has already put back, so nothing comes back twice.
///
/// A restored line and a restored component are the right side's own, bytes
/// and all, so every action that side took on them is already in them: they
/// come back once, whole, and the actions naming them are then let go rather
/// than written a second time.
#[derive(Default)]
pub(super) struct Restored<'a> {
    components: Vec<IcalComponentPath<'a>>,
    lines: Vec<(IcalComponentPath<'a>, String, usize)>,
}

impl<'a> Restored<'a> {
    /// Whether a component already put back holds this path.
    fn holds(&self, at: &IcalComponentPath<'a>) -> bool {
        self.components.iter().any(|held| at.0.starts_with(&held.0))
    }

    /// Take a line as put back, answering whether it is the first to claim it.
    fn claims(&mut self, at: &IcalPropPath<'a>) -> bool {
        let key = (at.component.clone(), at.name.to_ascii_uppercase(), at.index);

        if self.lines.contains(&key) {
            return false;
        }

        self.lines.push(key);

        true
    }
}

impl<'a> IcalMerge<'_, 'a> {
    /// Replay one right-side action onto the merged calendar.
    pub(super) fn apply(
        &self,
        merged: &mut IcalCst<'a>,
        op: &Op<'a>,
        shift: &Shift<'_>,
        restored: &mut Restored<'a>,
    ) {
        let host = match &op.action {
            IcalMergeAction::ComponentAdded { at } | IcalMergeAction::ComponentRemoved { at } => {
                at.parent()
            }
            _ => op.path().clone(),
        };

        // NOTE: a component put back is the right side's own, so this action
        // is already in it and replaying it would write it twice.
        if restored.holds(&host) {
            return;
        }

        if merged.at(&host).is_none() {
            // NOTE: the component is gone because the baseline side removed it
            // while the right side worked inside it. An act that writes
            // something brings it back, since keeping data beats losing it
            // silently; an act that only takes something away has nothing to
            // bring.
            if !op.action.is_removal() {
                self.restore(merged, &host, restored);
            }

            return;
        }

        match &op.action {
            IcalMergeAction::ComponentAdded { at } => {
                let Some(source) = self.right.at(at).cloned() else {
                    return;
                };
                let Some(target) = merged.at_mut(&at.parent()) else {
                    return;
                };

                target.items.push(IcalItem::Component(Box::new(source)));
            }
            IcalMergeAction::ComponentRemoved { at } => {
                let Some(target) = merged.at_mut(&at.parent()) else {
                    return;
                };

                if let Some(held) = target.component_position(at) {
                    target.items.remove(held);
                }
            }
            _ => self.apply_to_line(merged, op, shift, restored),
        }
    }

    /// Put back the highest component the baseline side removed out from under
    /// the right side's own act.
    ///
    /// It comes back as the right side wrote it, whole: the act that brought
    /// it back is in it, and so is every other act that side made inside it,
    /// which is why those are let go rather than replayed onto it.
    fn restore(
        &self,
        merged: &mut IcalCst<'a>,
        at: &IcalComponentPath<'a>,
        restored: &mut Restored<'a>,
    ) {
        let gone = at
            .ancestors()
            .chain(iter::once(at.clone()))
            .find(|path| merged.at(path).is_none());

        let Some(gone) = gone else {
            return;
        };
        let Some(source) = self.right.at(&gone).cloned() else {
            return;
        };
        let Some(target) = merged.at_mut(&gone.parent()) else {
            return;
        };

        target.items.push(IcalItem::Component(Box::new(source)));
        restored.components.push(gone);
    }

    /// Replay a property-level action onto the line it lands on.
    fn apply_to_line(
        &self,
        merged: &mut IcalCst<'a>,
        op: &Op<'a>,
        shift: &Shift<'_>,
        restored: &mut Restored<'a>,
    ) {
        let action = &op.action;

        let Some(at) = action.prop_path() else {
            return;
        };

        let source = self.written_line(self.right, op).map(IcalLine::terminated);

        let Some(component) = merged.at_mut(&at.component) else {
            return;
        };

        if let IcalMergeAction::PropAdded { .. } = action {
            let Some(source) = source else {
                return;
            };

            component.items.push(IcalItem::Prop(source));

            return;
        }

        let target = component.line_ordinal(at, shift.translate(at));

        if let IcalMergeAction::PropRemoved { .. } = action {
            if let Some(held) =
                target.and_then(|ordinal| component.line_position(&at.name, ordinal))
            {
                component.items.remove(held);
            }

            return;
        }

        let Some(source) = source else {
            return;
        };

        // NOTE: the line may be gone because the left side removed it while
        // the right side updated it. The update is what survives that
        // stand-off, so the line comes back rather than the update landing
        // nowhere. It comes back once: the restored line is the right side's
        // own, bytes and all, so every further action on that property is
        // already in it, and pushing again would leave one copy per action.
        let Some(line) = target.and_then(|ordinal| component.nth_line_mut(&at.name, ordinal))
        else {
            if restored.claims(at) {
                component.items.push(IcalItem::Prop(source));
            }

            return;
        };

        match action {
            IcalMergeAction::ValueChanged { .. } => line.value.clone_from(&source.value),
            // NOTE: A list is merged item by item rather than replaced, or the
            // right side's whole value would undo the left side's additions.
            IcalMergeAction::ValueItemAdded { item, .. } => {
                let mut items = line.merge_list();

                // NOTE: the list is written back only where the item really
                // joins it. Writing it back escapes every item afresh, so a
                // replay that changes nothing would still spell the left
                // side's own items the canonical way and churn bytes nobody
                // edited.
                if items.iter().any(|held| held == item) {
                    return;
                }

                items.push(item.to_string());
                line.set_merge_list(&items);
            }
            IcalMergeAction::ValueItemRemoved { item, .. } => {
                // NOTE: one item leaves, not every item equal to it. A list is
                // a multiset, so `a,a,b` losing one `a` keeps the other.
                let mut kept = line.merge_list();

                let Some(held) = kept.iter().position(|held| held == item) else {
                    return;
                };

                kept.remove(held);
                line.set_merge_list(&kept);
            }
            // NOTE: A parameter name may be written more than once on one line
            // (RFC 5545 3.2), so an action addresses the occurrence it named
            // rather than the first of that name.
            IcalMergeAction::ParamRemoved { .. } => {
                if let Slot::Param { name, at } = &op.slot
                    && let Some(held) = line.param_position(name, *at)
                {
                    line.params.remove(held);
                }
            }
            // NOTE: The parameter is copied off the source line rather than
            // re-encoded from the decoded action, so the side that wrote it
            // keeps its own spelling: a re-encoding would write the canonical
            // RFC 6868 form of a value the source spelled another way.
            IcalMergeAction::ParamAdded { .. } | IcalMergeAction::ParamChanged { .. } => {
                let Slot::Param { name, at } = &op.slot else {
                    return;
                };

                let Some(found) = source.param_position(name, *at) else {
                    return;
                };

                let written = source.params[found].clone();

                match line.param_position(name, *at) {
                    Some(held) => line.params[held] = written,
                    None => line.params.push(written),
                }
            }
            _ => {}
        }
    }
}

impl IcalLine<'_> {
    /// The items of the line's list value, an emptied list holding none.
    fn merge_list(&mut self) -> Vec<String> {
        let items: Vec<String> = IcalValueCursor { line: self }
            .list()
            .into_iter()
            .map(Cow::into_owned)
            .collect();

        // NOTE: The wire cannot tell an empty list from a list of one empty
        // item, so an emptied value reads back as a single empty item. Left
        // there, the next addition would keep it and write a leading comma
        // into the value.
        if items.iter().all(|item| item.is_empty()) {
            return Vec::new();
        }

        items
    }

    /// Replace the items of the line's list value.
    fn set_merge_list(&mut self, items: &[String]) {
        IcalValueCursor { line: self }.set_list(items);
    }
}
