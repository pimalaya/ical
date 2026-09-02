//! Algebraic laws, a completeness law and a differential reference for the
//! three-way merge.
//!
//! The merge is a pure function over bytes, so it can be held to laws rather
//! than to examples. Three things live here.
//!
//! The generator builds a base calendar and two edits derived from it, biased
//! so both sides frequently write into the same field: merging three unrelated
//! calendars would exercise nothing. What it builds carries the shapes the
//! merge reasons about, recurrence rules, overriding instances addressed by
//! `RECURRENCE-ID`, `VALARM` children, `ATTENDEE` lines with parameters, zoned
//! and floating starts, and a folded line nobody edits.
//!
//! The [`model`] module projects a calendar onto the fields two sides can
//! contest, the same granularity the merge itself works at: whether a component
//! exists, whether a property exists, the value of a non-list property, the
//! presence of one item of a list value, and the value of one parameter. Every
//! law is stated over those fields.
//!
//! The [`reference`] module is a second merge, written from the spec by plain
//! set operations over that projection, with no byte preservation and no clever
//! matching. Where the two disagree one of them is wrong, and the disagreement
//! is the finding.

#![cfg(feature = "parser")]

use std::collections::{BTreeMap, BTreeSet};

use ical::tree::{
    cst::IcalCst,
    merge::{IcalMerge, IcalMergeReport},
};
use proptest::{
    prelude::*,
    strategy::ValueTree,
    test_runner::{Config, TestRunner},
};

use crate::{
    generator::scenario,
    model::{FieldKey, FieldSlot, IcalModel},
};

/// How many cases each law runs, overridable with `PROPTEST_CASES` for a
/// longer soak.
fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|held| held.parse().ok())
        .unwrap_or(512)
}

/// Merge three calendars given as wire bytes.
fn merge<'a>(base: &'a [u8], left: &'a [u8], right: &'a [u8]) -> Option<IcalMergeReport<'a>> {
    let base = Box::leak(Box::new(IcalCst::parse(base).ok()?));
    let left = Box::leak(Box::new(IcalCst::parse(left).ok()?));
    let right = Box::leak(Box::new(IcalCst::parse(right).ok()?));

    Some(IcalMerge { base, left, right }.merge())
}

/// The merged calendar's bytes.
fn bytes(report: &IcalMergeReport<'_>) -> Vec<u8> {
    report.merged.to_bytes()
}

/// The four models a law is stated over, plus the report that produced the
/// fourth.
struct Merged<'a> {
    /// The common ancestor.
    base: IcalModel,
    /// One side.
    left: IcalModel,
    /// The other side.
    right: IcalModel,
    /// What the merge produced.
    merged: IcalModel,
    /// The merge's own report.
    report: IcalMergeReport<'a>,
}

/// Merge three calendars and project all four onto the field model.
fn run<'a>(base: &'a [u8], left: &'a [u8], right: &'a [u8]) -> Option<Merged<'a>> {
    let report = merge(base, left, right)?;

    let base = IcalCst::parse(base).ok()?;
    let version = base.version();
    let left = IcalCst::parse(left).ok()?;
    let right = IcalCst::parse(right).ok()?;

    Some(Merged {
        base: model::of(&base, version),
        left: model::of(&left, version),
        right: model::of(&right, version),
        merged: model::of(&report.merged, version),
        report,
    })
}

/// The projection of a calendar onto the fields two sides can contest.
mod model {
    use std::collections::{BTreeMap, BTreeSet};

    use ical::{
        param::IcalParam,
        prop::{IcalPropKind, IcalPropName},
        tree::{
            cst::{IcalCst, IcalItem},
            line::IcalLine,
            merge::{IcalComponentPath, IcalMergeAction, IcalMergeReport, IcalPropPath},
        },
        value::IcalValue,
        version::IcalVersion,
    };

    /// A calendar as the fields two sides can contest.
    pub type IcalModel = BTreeMap<FieldKey, String>;

    /// One field of a calendar: the smallest unit the merge decides about.
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct FieldKey {
        /// The component holding it, root first, each step a name and the
        /// identity that tells it from its same-named siblings.
        pub component: Vec<(String, String)>,
        /// The property name, uppercase, empty for a component-level field.
        pub prop: String,
        /// What tells it from the component's other properties of that name:
        /// the identity iCalendar gives the property, or its position written
        /// after a hash where it has none.
        pub at: String,
        /// Which part of the property the field is.
        pub slot: FieldSlot,
    }

    /// Which part of a property a field is.
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum FieldSlot {
        /// The component exists.
        Component,
        /// The property exists.
        Prop,
        /// The whole value of a property that does not decode to a list.
        Value,
        /// One item of a list value, present or absent.
        Item(String),
        /// One parameter, by uppercase name.
        Param(String),
    }

    /// Project a calendar onto its fields.
    pub fn of(cst: &IcalCst<'_>, version: IcalVersion) -> IcalModel {
        let mut out = IcalModel::new();
        walk(cst, Vec::new(), version, &mut out);
        out
    }

    /// Collect one component's fields and everything nested in it.
    fn walk(
        cst: &IcalCst<'_>,
        path: Vec<(String, String)>,
        version: IcalVersion,
        out: &mut IcalModel,
    ) {
        out.insert(
            FieldKey {
                component: path.clone(),
                prop: String::new(),
                at: String::new(),
                slot: FieldSlot::Component,
            },
            String::new(),
        );

        let props: Vec<&IcalLine<'_>> = lines(cst).collect();

        for (at, line) in props.iter().enumerate() {
            let name = line.name.get().to_ascii_uppercase();
            let index = props[..at]
                .iter()
                .filter(|held| held.name.get().eq_ignore_ascii_case(&name))
                .count();
            let held = identity(&props, at, index);

            let field = |slot| FieldKey {
                component: path.clone(),
                prop: name.clone(),
                at: held.clone(),
                slot,
            };

            out.insert(field(FieldSlot::Prop), String::new());

            let prop = line.decode(version);

            match &prop.value {
                IcalValue::TextList(list) => {
                    for item in &list.0 {
                        out.insert(field(FieldSlot::Item(item.to_string())), String::new());
                    }
                }
                IcalValue::DateTimeList(list) => {
                    for item in &list.0 {
                        out.insert(field(FieldSlot::Item(item.to_string())), String::new());
                    }
                }
                value => {
                    out.insert(field(FieldSlot::Value), format!("{value:?}"));
                }
            }

            for param in &prop.params {
                let values = format!("{param:?}");
                out.insert(field(FieldSlot::Param(param_name(param))), values);
            }
        }

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
            nested.push((name, key(child, ordinal)));

            walk(child, nested, version, out);
        }
    }

    /// What tells a line from its component's other properties of that name,
    /// the merge's own rule: the value of a property iCalendar identifies by
    /// what it names, unless a same-named sibling repeats it, and its position
    /// after a hash for everything else.
    pub fn identity(lines: &[&IcalLine<'_>], at: usize, index: usize) -> String {
        let held = value_of(lines[at]);
        let name = lines[at].name.get();

        let twice = held.is_some()
            && lines.iter().enumerate().any(|(other, line)| {
                other != at && line.name.get().eq_ignore_ascii_case(name) && value_of(line) == held
            });

        match held {
            Some(held) if !twice => held,
            _ => format!("#{index}"),
        }
    }

    /// The whole raw value of a line iCalendar identifies by what it names.
    fn value_of(line: &IcalLine<'_>) -> Option<String> {
        let identified = matches!(
            IcalPropName::from(std::borrow::Cow::Owned(line.name.get().to_owned())),
            IcalPropName::Kind(
                IcalPropKind::Attendee
                    | IcalPropKind::Attach
                    | IcalPropKind::RelatedTo
                    | IcalPropKind::Conference
                    | IcalPropKind::Image
            )
        );

        identified.then(|| line.value.to_string())
    }

    /// The property lines of a component, in source order, the envelope
    /// keywords left out as the merge leaves them out.
    fn lines<'c, 'a>(cst: &'c IcalCst<'a>) -> impl Iterator<Item = &'c IcalLine<'a>> {
        cst.items
            .iter()
            .filter_map(|item| match item {
                IcalItem::Prop(line) => Some(line),
                _ => None,
            })
            .filter(|line| {
                let name = line.name.get();

                !name.eq_ignore_ascii_case("BEGIN") && !name.eq_ignore_ascii_case("END")
            })
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

    /// A component's identity among its same-named siblings, the merge's own
    /// rule: its `UID`, with the `RECURRENCE-ID` after a solidus when it
    /// overrides one instance, or its position when it carries no `UID`.
    fn key(cst: &IcalCst<'_>, ordinal: usize) -> String {
        let Some(uid) = raw(cst, "UID") else {
            return ordinal.to_string();
        };

        match raw(cst, "RECURRENCE-ID") {
            Some(id) => format!("{uid}/{id}"),
            None => uid,
        }
    }

    /// The raw text of a component's first property of this name.
    fn raw(cst: &IcalCst<'_>, name: &str) -> Option<String> {
        lines(cst)
            .find(|line| line.name.get().eq_ignore_ascii_case(name))
            .map(|line| line.raw_value_str().into_owned())
    }

    /// A parameter's name, the key the merge matches parameters on.
    pub fn param_name(param: &IcalParam<'_>) -> String {
        match param {
            IcalParam::Unknown { name, .. } => name.to_ascii_uppercase(),
            known => known
                .kind()
                .map(|kind| kind.to_ascii_uppercase())
                .unwrap_or_default(),
        }
    }

    /// What a reported path says tells its property from its same-named
    /// siblings, as the model spells it.
    pub fn at_of(at: &IcalPropPath<'_>) -> String {
        match &at.identity {
            Some(identity) => identity.to_string(),
            None => format!("#{}", at.index),
        }
    }

    /// A component path as the model spells it.
    pub fn component_of(path: &IcalComponentPath<'_>) -> Vec<(String, String)> {
        path.0
            .iter()
            .map(|step| (step.name.to_string(), step.key.to_string()))
            .collect()
    }

    /// The fields one reported action is about, among the fields that exist in
    /// any of the four calendars.
    pub fn fields_of(action: &IcalMergeAction<'_>, universe: &BTreeSet<FieldKey>) -> Vec<FieldKey> {
        let prop_fields = |at: &IcalPropPath<'_>| {
            let component = component_of(&at.component);
            let prop = at.name.to_ascii_uppercase();
            let held = at_of(at);

            universe
                .iter()
                .filter(|key| key.component == component && key.prop == prop && key.at == held)
                .cloned()
                .collect::<Vec<_>>()
        };

        let one = |at: &IcalPropPath<'_>, slot: FieldSlot| {
            vec![FieldKey {
                component: component_of(&at.component),
                prop: at.name.to_ascii_uppercase(),
                at: at_of(at),
                slot,
            }]
        };

        match action {
            IcalMergeAction::ComponentAdded { at } | IcalMergeAction::ComponentRemoved { at } => {
                let component = component_of(at);

                universe
                    .iter()
                    .filter(|key| key.component.starts_with(&component))
                    .cloned()
                    .collect()
            }
            IcalMergeAction::PropAdded { at, .. } | IcalMergeAction::PropRemoved { at, .. } => {
                prop_fields(at)
            }
            IcalMergeAction::ValueChanged { at, .. } => one(at, FieldSlot::Value),
            IcalMergeAction::ValueItemAdded { at, item }
            | IcalMergeAction::ValueItemRemoved { at, item } => {
                one(at, FieldSlot::Item(item.to_string()))
            }
            IcalMergeAction::ParamAdded { at, param }
            | IcalMergeAction::ParamRemoved { at, param } => {
                one(at, FieldSlot::Param(param_name(param)))
            }
            IcalMergeAction::ParamChanged { at, new, .. } => {
                one(at, FieldSlot::Param(param_name(new)))
            }
        }
    }

    /// The fields the report names as contested, that is the ones a divergence
    /// was reported about.
    ///
    /// Both halves of the pair count. Either side's action may be the one that
    /// did not land, and where a component removal met an edit inside it, the
    /// removal is the loser and the left half is where the report names it.
    ///
    /// A recurrence conflict is deliberately not counted: it refuses nothing,
    /// so it can never be the reason a change failed to land.
    pub fn contested(
        report: &IcalMergeReport<'_>,
        universe: &BTreeSet<FieldKey>,
    ) -> BTreeSet<FieldKey> {
        let mut out = BTreeSet::new();

        for conflict in &report.conflicts {
            let ical::tree::merge::IcalMergeReason::Divergent(left) = &conflict.left else {
                continue;
            };

            out.extend(fields_of(left, universe));
            out.extend(fields_of(&conflict.right, universe));
        }

        out
    }

    /// The address a conflict is about: the component holding the property,
    /// its name and which of the component's same-named properties it is. A
    /// conflict about a whole component carries the empty name.
    ///
    /// This is coarser than a field on purpose. The same collision is reported
    /// through whichever action the diff happened to produce, and a removal
    /// names a whole property where an update names only its value, so
    /// comparing two reports field by field would count a difference in
    /// spelling as a difference in substance.
    pub type Address = (Vec<(String, String)>, String, String);

    /// The addresses the report names as contested.
    pub fn contested_addresses(report: &IcalMergeReport<'_>) -> BTreeSet<Address> {
        report
            .conflicts
            .iter()
            .filter(|conflict| {
                !matches!(
                    conflict.left,
                    ical::tree::merge::IcalMergeReason::Recurrence(_)
                )
            })
            .map(|conflict| address_of(&conflict.right))
            .collect()
    }

    /// The address one action is about.
    pub fn address_of(action: &IcalMergeAction<'_>) -> Address {
        match action {
            IcalMergeAction::ComponentAdded { at } | IcalMergeAction::ComponentRemoved { at } => {
                (component_of(at), String::new(), String::new())
            }
            IcalMergeAction::PropAdded { at, .. }
            | IcalMergeAction::PropRemoved { at, .. }
            | IcalMergeAction::ValueChanged { at, .. }
            | IcalMergeAction::ValueItemAdded { at, .. }
            | IcalMergeAction::ValueItemRemoved { at, .. }
            | IcalMergeAction::ParamAdded { at, .. }
            | IcalMergeAction::ParamRemoved { at, .. }
            | IcalMergeAction::ParamChanged { at, .. } => (
                component_of(&at.component),
                at.name.to_ascii_uppercase(),
                at_of(at),
            ),
        }
    }
}

/// A second merge, written from the spec rather than from the implementation.
///
/// It knows nothing about bytes, folding, property order or line identity. It
/// diffs the field projection of each side against the base with plain set
/// operations and reconciles by the documented rules: a field only one side
/// touched is taken from that side, an update beats a removal whichever side it
/// came from, and where both sides wrote a different value the left side's
/// survives. Its only job is to disagree with the real merge where the real
/// merge is wrong.
mod reference {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::model::{Address, FieldKey, FieldSlot, IcalModel};

    /// What the reference merge produced.
    pub struct Reference {
        /// The merged fields.
        pub merged: IcalModel,
        /// The addresses it reports as contested.
        pub contested: BTreeSet<Address>,
    }

    /// The properties one calendar puts in each of its components.
    type Props = BTreeMap<Vec<(String, String)>, BTreeSet<(String, String)>>;

    /// One calendar, arranged the way the reconcile reads it.
    ///
    /// The reconcile asks the same three questions about every component and
    /// every property, and answering them by sweeping the model each time turns
    /// a merge into a quadratic scan of the calendar.
    struct Side {
        /// The components it holds.
        paths: BTreeSet<Vec<(String, String)>>,
        /// The properties it puts in each component.
        props: Props,
        /// The fields of each property, existence included.
        fields: BTreeMap<Address, Vec<(FieldKey, String)>>,
    }

    impl Side {
        /// Arrange one calendar.
        fn of(model: &IcalModel) -> Self {
            let mut paths = BTreeSet::new();
            let mut props = Props::new();
            let mut fields: BTreeMap<Address, Vec<(FieldKey, String)>> = BTreeMap::new();

            for (key, value) in model {
                if key.slot == FieldSlot::Component {
                    paths.insert(key.component.clone());
                    continue;
                }

                let address = (key.component.clone(), key.prop.clone(), key.at.clone());

                if key.slot == FieldSlot::Prop {
                    props
                        .entry(key.component.clone())
                        .or_default()
                        .insert((key.prop.clone(), key.at.clone()));
                }

                fields
                    .entry(address)
                    .or_default()
                    .push((key.clone(), value.clone()));
            }

            Self {
                paths,
                props,
                fields,
            }
        }

        /// Whether it holds a component.
        fn has_component(&self, path: &[(String, String)]) -> bool {
            self.paths.contains(path)
        }

        /// Whether it holds a property.
        fn has_prop(&self, address: &Address) -> bool {
            self.fields
                .get(address)
                .is_some_and(|held| held.iter().any(|(key, _)| key.slot == FieldSlot::Prop))
        }

        /// One property as it holds it, existence included.
        fn fields(&self, address: &Address) -> &[(FieldKey, String)] {
            self.fields.get(address).map(Vec::as_slice).unwrap_or(&[])
        }
    }

    /// Reconcile two field projections against their base.
    pub fn merge(base: &IcalModel, left: &IcalModel, right: &IcalModel) -> Reference {
        let subtree = |model: &IcalModel, path: &[(String, String)]| {
            model
                .iter()
                .filter(|(key, _)| key.component.starts_with(path))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<BTreeMap<_, _>>()
        };

        let same = |path: &[(String, String)]| subtree(left, path) == subtree(right, path);

        let base = Side::of(base);
        let left = Side::of(left);
        let right = Side::of(right);

        let mut merged = IcalModel::new();
        let mut contested = BTreeSet::new();
        let mut wholesale: BTreeSet<Vec<(String, String)>> = BTreeSet::new();

        let mut paths: BTreeSet<&Vec<(String, String)>> = BTreeSet::new();
        paths.extend(&base.paths);
        paths.extend(&left.paths);
        paths.extend(&right.paths);

        for path in paths {
            let in_base = base.has_component(path);
            let in_left = left.has_component(path);
            let in_right = right.has_component(path);

            let alive = match (in_base, in_left, in_right) {
                (false, ..) => in_left || in_right,
                (true, ..) => in_left && in_right,
            };

            if !alive {
                continue;
            }

            // Both sides added one component, so the whole subtree is the
            // left side's rather than a field-by-field blend of two components
            // that never shared an ancestor. Two sides that added the same
            // component agreed, and agreement is not a collision.
            if !in_base && in_left && in_right && !same(path) {
                contested.insert((path.clone(), String::new(), String::new()));
                wholesale.insert(path.clone());
            }

            merged.insert(
                FieldKey {
                    component: path.clone(),
                    prop: String::new(),
                    at: String::new(),
                    slot: FieldSlot::Component,
                },
                String::new(),
            );

            let taken_whole = wholesale.iter().any(|held| path.starts_with(held));

            let mut props: BTreeSet<&(String, String)> = BTreeSet::new();

            for side in [&base, &left, &right] {
                if let Some(held) = side.props.get(path) {
                    props.extend(held);
                }
            }

            for (name, at) in props {
                let address = (path.clone(), name.clone(), at.clone());

                if !taken_whole {
                    reconcile(&base, &left, &right, &address, &mut merged, &mut contested);

                    continue;
                }

                for (key, value) in left.fields(&address) {
                    merged.insert(key.clone(), value.clone());
                }
            }
        }

        Reference { merged, contested }
    }

    /// Reconcile one property of one component, field by field.
    fn reconcile(
        base: &Side,
        left: &Side,
        right: &Side,
        address: &Address,
        merged: &mut IcalModel,
        contested: &mut BTreeSet<Address>,
    ) {
        let in_base = base.has_prop(address);
        let in_left = left.has_prop(address);
        let in_right = right.has_prop(address);

        let changed_left = base.fields(address) != left.fields(address);
        let changed_right = base.fields(address) != right.fields(address);

        // A removal met an update: the update survives whichever side it came
        // from, and the line it survives as is the updating side's.
        let restored = match (in_base, in_left, in_right) {
            (true, false, true) if changed_right => Some(right),
            (true, true, false) if changed_left => Some(left),
            _ => None,
        };

        if let Some(held) = restored {
            contested.insert(address.clone());

            for (key, value) in held.fields(address) {
                merged.insert(key.clone(), value.clone());
            }

            return;
        }

        let alive = match (in_base, in_left, in_right) {
            (false, ..) => in_left || in_right,
            (true, ..) => in_left && in_right,
        };

        if !alive {
            return;
        }

        let mut slots: BTreeSet<&FieldSlot> = BTreeSet::new();

        for side in [base, left, right] {
            slots.extend(side.fields(address).iter().map(|(key, _)| &key.slot));
        }

        for slot in slots {
            let key = FieldKey {
                component: address.0.clone(),
                prop: address.1.clone(),
                at: address.2.clone(),
                slot: slot.clone(),
            };

            let of = |side: &Side| {
                side.fields(address)
                    .iter()
                    .find(|(held, _)| held.slot == *slot)
                    .map(|(_, value)| value.clone())
            };

            let (b, l, r) = (of(base), of(left), of(right));

            let winner = if r == b {
                l
            } else if l == b {
                r
            } else if l == r {
                l
            } else {
                contested.insert(address.clone());

                // NOTE: The left side is git's ours, so where both sides wrote
                // a different value the left one stands. A side holding no
                // value removed the field, and an update beats a removal.
                if l.is_none() { r } else { l }
            };

            if let Some(value) = winner {
                merged.insert(key, value);
            }
        }
    }
}

/// The calendars the laws are stated over, and the edits that derive two sides
/// from one base.
///
/// A calendar is built as a component tree and rendered to wire bytes, so an
/// untouched property renders to the same bytes on both sides and byte
/// preservation is checkable without guessing. Edits address the base by
/// position, and a scenario draws most of its edits from a shared pool so both
/// sides land on the same field often enough for the collision rules to be
/// exercised at all.
mod generator {
    use proptest::prelude::*;

    /// One component of a generated calendar.
    #[derive(Clone, Debug)]
    pub struct Comp {
        /// The component name, uppercase.
        pub name: String,
        /// Its property lines, in order.
        pub props: Vec<Prop>,
        /// Its nested components, in order.
        pub children: Vec<Comp>,
    }

    /// One property line of a generated calendar.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Prop {
        /// The property name, uppercase.
        pub name: String,
        /// Its parameters, in order.
        pub params: Vec<(String, String)>,
        /// Its raw value.
        pub value: String,
        /// Whether the renderer folds it, so a folded line nobody edits is
        /// there to check byte preservation against.
        pub folded: bool,
    }

    impl Prop {
        /// A parameterless property.
        pub fn new(name: &str, value: &str) -> Self {
            Self {
                name: name.to_owned(),
                params: Vec::new(),
                value: value.to_owned(),
                folded: false,
            }
        }

        /// The same with parameters.
        pub fn with(name: &str, params: &[(&str, &str)], value: &str) -> Self {
            Self {
                params: params
                    .iter()
                    .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                    .collect(),
                ..Self::new(name, value)
            }
        }
    }

    /// One change a side makes to the base.
    ///
    /// Every variant addresses the base rather than the calendar being built,
    /// by an index taken modulo the number of candidates, so the two sides can
    /// be pointed at one target without knowing what the base holds.
    #[derive(Clone, Debug)]
    pub enum Edit {
        /// Write a different value into an existing property.
        SetValue {
            /// Which editable property.
            slot: usize,
            /// Which of the generator's replacement values.
            seed: usize,
        },
        /// Remove an existing property.
        RemoveProp {
            /// Which editable property.
            slot: usize,
        },
        /// Add a property to a component.
        AddProp {
            /// Which component.
            comp: usize,
            /// Which of the generator's addable properties.
            seed: usize,
        },
        /// Add one item to a list value.
        AddListItem {
            /// Which list-valued property.
            slot: usize,
            /// Which of the generator's items.
            seed: usize,
        },
        /// Remove the first item of a list value.
        RemoveListItem {
            /// Which list-valued property.
            slot: usize,
        },
        /// Write a different value into an existing parameter.
        SetParam {
            /// Which property carrying a parameter.
            slot: usize,
            /// Which of the generator's parameter values.
            seed: usize,
        },
        /// Remove an existing parameter.
        RemoveParam {
            /// Which property carrying a parameter.
            slot: usize,
        },
        /// Add a parameter to a property.
        AddParam {
            /// Which editable property.
            slot: usize,
            /// Which of the generator's parameter values.
            seed: usize,
        },
        /// Add a `VALARM` to a component.
        AddAlarm {
            /// Which component.
            comp: usize,
            /// Which of the generator's alarms.
            seed: usize,
        },
        /// Remove a whole component other than the calendar itself.
        RemoveComp {
            /// Which nested component.
            comp: usize,
        },
    }

    /// A base calendar and two edits of it.
    #[derive(Clone, Debug)]
    pub struct Scenario {
        /// The common ancestor.
        pub base: Comp,
        /// What the left side did.
        pub left: Vec<Edit>,
        /// What the right side did.
        pub right: Vec<Edit>,
    }

    impl Scenario {
        /// The base, as wire bytes.
        pub fn base_bytes(&self) -> Vec<u8> {
            render(&self.base)
        }

        /// The left side, as wire bytes.
        pub fn left_bytes(&self) -> Vec<u8> {
            render(&apply_all(&self.base, &self.left))
        }

        /// The right side, as wire bytes.
        pub fn right_bytes(&self) -> Vec<u8> {
            render(&apply_all(&self.base, &self.right))
        }

        /// The rendered lines of the base that neither side changed, so the
        /// merged calendar has to carry them byte for byte, folds included.
        ///
        /// A line counts as untouched when its rendered bytes are still in
        /// both sides. Reading that off the two renderings rather than off the
        /// edit list is what makes it trustworthy: an edit can miss its target
        /// or land on a neighbour, and the bookkeeping would then say a line
        /// was untouched when the side that was meant to touch it changed
        /// something else instead. Lines the base holds twice are left out,
        /// since one of the two could go while the bytes stayed.
        pub fn untouched_lines(&self) -> Vec<String> {
            let base = String::from_utf8(self.base_bytes()).expect("valid UTF-8");
            let left = String::from_utf8(self.left_bytes()).expect("valid UTF-8");
            let right = String::from_utf8(self.right_bytes()).expect("valid UTF-8");

            let unique = |text: &str| {
                base.matches(text).count() == 1
                    && left.matches(text).count() == 1
                    && right.matches(text).count() == 1
            };

            all_props(&self.base)
                .into_iter()
                .filter(|(path, prop)| {
                    if !unique(&render_prop(prop)) {
                        return false;
                    }

                    // The same bytes elsewhere are not the same line, so the
                    // component holding it has to be the one that survived,
                    // whole and unedited, on both sides.
                    match at(&self.base, path) {
                        None => false,
                        Some(_) if path.is_empty() => true,
                        Some(comp) => {
                            let text = String::from_utf8(render(comp)).expect("valid UTF-8");

                            left.contains(&text) && right.contains(&text)
                        }
                    }
                })
                .map(|(_, prop)| render_prop(&prop))
                .collect()
        }
    }

    /// The candidate an index names, taken modulo how many there are.
    fn pick<T: Clone>(candidates: &[T], at: usize) -> Option<T> {
        candidates.get(at % candidates.len().max(1)).cloned()
    }

    /// Every property of a calendar, with the path of the component holding
    /// it, in a stable depth-first order.
    fn all_props(comp: &Comp) -> Vec<(Vec<usize>, Prop)> {
        let mut out = Vec::new();
        collect_props(comp, Vec::new(), &mut out);
        out
    }

    /// The properties an edit may address.
    ///
    /// The identity properties are held back: a side rewriting a `UID` or a
    /// `RECURRENCE-ID` does not edit a component, it replaces it with a
    /// different one, which is a different scenario from the ones these laws
    /// are about.
    fn editable_props(comp: &Comp) -> Vec<(Vec<usize>, Prop)> {
        all_props(comp)
            .into_iter()
            .filter(|(_, prop)| {
                !matches!(
                    prop.name.as_str(),
                    "UID" | "RECURRENCE-ID" | "VERSION" | "PRODID"
                )
            })
            .collect()
    }

    /// The editable properties carrying a comma-separated list value.
    fn list_props(comp: &Comp) -> Vec<(Vec<usize>, Prop)> {
        editable_props(comp)
            .into_iter()
            .filter(|(_, prop)| is_list(&prop.name))
            .collect()
    }

    /// The editable properties carrying at least one parameter.
    fn param_props(comp: &Comp) -> Vec<(Vec<usize>, Prop)> {
        editable_props(comp)
            .into_iter()
            .filter(|(_, prop)| !prop.params.is_empty())
            .collect()
    }

    /// Collect one component's properties and its children's.
    fn collect_props(comp: &Comp, path: Vec<usize>, out: &mut Vec<(Vec<usize>, Prop)>) {
        for prop in &comp.props {
            out.push((path.clone(), prop.clone()));
        }

        for (at, child) in comp.children.iter().enumerate() {
            let mut nested = path.clone();
            nested.push(at);
            collect_props(child, nested, out);
        }
    }

    /// Every component of a calendar, the calendar itself first.
    fn comps_of(comp: &Comp) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        collect_comps(comp, Vec::new(), &mut out);
        out
    }

    /// The same, without the calendar itself, which nothing removes.
    fn nested_comps(comp: &Comp) -> Vec<Vec<usize>> {
        comps_of(comp)
            .into_iter()
            .filter(|path| !path.is_empty())
            .collect()
    }

    /// Collect one component's path and its children's.
    fn collect_comps(comp: &Comp, path: Vec<usize>, out: &mut Vec<Vec<usize>>) {
        out.push(path.clone());

        for (at, child) in comp.children.iter().enumerate() {
            let mut nested = path.clone();
            nested.push(at);
            collect_comps(child, nested, out);
        }
    }

    /// Apply a side's edits to the base.
    pub fn apply_all(base: &Comp, edits: &[Edit]) -> Comp {
        let mut held = base.clone();

        for edit in edits {
            apply(&mut held, base, edit);
        }

        held
    }

    /// The values an edit writes, indexed by its seed.
    const VALUES: [&str; 4] = ["one", "two", "three", "four"];
    /// The parameter values an edit writes, indexed by its seed.
    const PARAMS: [&str; 4] = ["ACCEPTED", "DECLINED", "TENTATIVE", "DELEGATED"];
    /// The list items an edit adds, indexed by its seed.
    const ITEMS: [&str; 4] = ["alpha", "beta", "gamma", "delta"];

    /// Apply one edit, addressing the base rather than the already-edited
    /// calendar so two edits of one side never renumber each other.
    fn apply(held: &mut Comp, base: &Comp, edit: &Edit) {
        match edit {
            Edit::SetValue { slot, seed } => {
                let Some((path, prop)) = pick(&editable_props(base), *slot) else {
                    return;
                };

                let value = replacement(&prop.value, VALUES[seed % VALUES.len()]);
                edit_prop(held, base, &path, &prop, &mut |prop| {
                    prop.value = value.clone()
                });
            }
            Edit::RemoveProp { slot } => {
                let Some((path, prop)) = pick(&editable_props(base), *slot) else {
                    return;
                };

                let Some(comp) = at_mut(held, base, &path) else {
                    return;
                };

                if let Some(at) = comp.props.iter().position(|held| *held == prop) {
                    comp.props.remove(at);
                }
            }
            Edit::AddProp { comp, seed } => {
                let Some(path) = pick(&comps_of(base), *comp) else {
                    return;
                };

                let added = match seed % 4 {
                    0 => Prop::new("COMMENT", VALUES[seed % VALUES.len()]),
                    1 => Prop::new("LOCATION", VALUES[seed % VALUES.len()]),
                    2 => Prop::with(
                        "ATTENDEE",
                        &[("PARTSTAT", "NEEDS-ACTION")],
                        "mailto:bob@example.com",
                    ),
                    _ => Prop::new("X-PIMALAYA-NOTE", VALUES[seed % VALUES.len()]),
                };

                if let Some(comp) = at_mut(held, base, &path) {
                    comp.props.push(added);
                }
            }
            Edit::AddListItem { slot, seed } => {
                let Some((path, prop)) = pick(&list_props(base), *slot) else {
                    return;
                };

                let item = ITEMS[seed % ITEMS.len()].to_owned();
                edit_prop(held, base, &path, &prop, &mut |prop| {
                    if !prop.value.split(',').any(|held| held == item) {
                        prop.value = format!("{},{item}", prop.value);
                    }
                });
            }
            Edit::RemoveListItem { slot } => {
                let Some((path, prop)) = pick(&list_props(base), *slot) else {
                    return;
                };

                edit_prop(held, base, &path, &prop, &mut |prop| {
                    let mut items: Vec<&str> = prop.value.split(',').collect();

                    if items.len() > 1 {
                        items.remove(0);
                        prop.value = items.join(",");
                    }
                });
            }
            Edit::SetParam { slot, seed } => {
                let Some((path, prop)) = pick(&param_props(base), *slot) else {
                    return;
                };

                let value = PARAMS[seed % PARAMS.len()].to_owned();
                edit_prop(held, base, &path, &prop, &mut |prop| {
                    if let Some(param) = prop.params.first_mut() {
                        param.1 = value.clone();
                    }
                });
            }
            Edit::RemoveParam { slot } => {
                let Some((path, prop)) = pick(&param_props(base), *slot) else {
                    return;
                };

                edit_prop(held, base, &path, &prop, &mut |prop| {
                    if !prop.params.is_empty() {
                        prop.params.remove(0);
                    }
                });
            }
            Edit::AddParam { slot, seed } => {
                let Some((path, prop)) = pick(&editable_props(base), *slot) else {
                    return;
                };

                let value = VALUES[seed % VALUES.len()].to_owned();
                edit_prop(held, base, &path, &prop, &mut |prop| {
                    if !prop.params.iter().any(|(name, _)| name == "X-LABEL") {
                        prop.params.push(("X-LABEL".to_owned(), value.clone()));
                    }
                });
            }
            Edit::AddAlarm { comp, seed } => {
                let Some(path) = pick(&comps_of(base), *comp) else {
                    return;
                };

                let alarm = Comp {
                    name: "VALARM".to_owned(),
                    props: vec![
                        Prop::new("ACTION", "DISPLAY"),
                        Prop::new("TRIGGER", if seed % 2 == 0 { "-PT15M" } else { "-PT30M" }),
                        Prop::new("DESCRIPTION", VALUES[seed % VALUES.len()]),
                    ],
                    children: Vec::new(),
                };

                if let Some(comp) = at_mut(held, base, &path) {
                    comp.children.push(alarm);
                }
            }
            Edit::RemoveComp { comp } => {
                let Some(path) = pick(&nested_comps(base), *comp) else {
                    return;
                };

                let Some((last, parent)) = path.split_last() else {
                    return;
                };

                let Some(wanted) = at(base, parent)
                    .map(identities)
                    .and_then(|held| held.into_iter().nth(*last))
                else {
                    return;
                };

                let Some(comp) = at_mut(held, base, parent) else {
                    return;
                };

                if let Some(at) = identities(comp).into_iter().position(|held| held == wanted) {
                    comp.children.remove(at);
                }
            }
        }
    }

    /// Whether a property name carries a comma-separated list.
    fn is_list(name: &str) -> bool {
        matches!(name, "CATEGORIES" | "RESOURCES" | "EXDATE" | "RDATE")
    }

    /// A replacement value that keeps the shape of the one it replaces, so a
    /// date stays a date and a recurrence rule stays a rule.
    ///
    /// The rules alternate between `COUNT` and `UNTIL`, and the dates move by
    /// a day, so a side can move the ground an override stood on.
    fn replacement(old: &str, seed: &str) -> String {
        if old.starts_with("FREQ=") {
            return match seed {
                "one" => "FREQ=DAILY;COUNT=5".to_owned(),
                "two" => "FREQ=WEEKLY;UNTIL=20260401T090000Z".to_owned(),
                "three" => "FREQ=MONTHLY;COUNT=3".to_owned(),
                _ => "FREQ=WEEKLY;INTERVAL=2".to_owned(),
            };
        }

        if old.len() >= 15 && old.as_bytes()[8] == b'T' {
            let day = match seed {
                "one" => "06",
                "two" => "07",
                "three" => "08",
                _ => "09",
            };

            return format!("{}{day}{}", &old[..6], &old[8..]);
        }

        if old.contains(',') {
            return format!("{seed},{seed}-bis");
        }

        format!("{old} ({seed})")
    }

    /// Rewrite the property a path and a base value name, so an edit finds
    /// its target whatever an earlier edit of the same side did.
    fn edit_prop(
        held: &mut Comp,
        base: &Comp,
        path: &[usize],
        prop: &Prop,
        change: &mut dyn FnMut(&mut Prop),
    ) {
        let Some(comp) = at_mut(held, base, path) else {
            return;
        };

        let Some(held) = comp
            .props
            .iter_mut()
            .find(|held| held.name == prop.name && held.value == prop.value)
        else {
            return;
        };

        change(held);
    }

    /// The component a path of base child positions names, found by the
    /// identity the merge itself uses rather than by position, so an edit
    /// that removed an earlier sibling does not send the next one astray.
    fn at_mut<'c>(held: &'c mut Comp, base: &Comp, path: &[usize]) -> Option<&'c mut Comp> {
        let Some((last, above)) = path.split_last() else {
            return Some(held);
        };

        let wanted = identities(at(base, above)?).into_iter().nth(*last)?;
        let held = at_mut(held, base, above)?;
        let at = identities(held)
            .into_iter()
            .position(|held| held == wanted)?;

        held.children.get_mut(at)
    }

    /// The component a path of base child positions names, in the base.
    fn at<'c>(comp: &'c Comp, path: &[usize]) -> Option<&'c Comp> {
        let mut held = comp;

        for step in path {
            held = held.children.get(*step)?;
        }

        Some(held)
    }

    /// The identity of every child of a component, in order: its name, then
    /// its `UID` with its `RECURRENCE-ID` after a solidus, or its position
    /// among its same-named siblings when it carries no `UID`. This is the
    /// identity the merge itself matches components on.
    fn identities(comp: &Comp) -> Vec<(String, String)> {
        let mut seen: Vec<(String, usize)> = Vec::new();
        let mut out = Vec::new();

        for child in &comp.children {
            let ordinal = match seen.iter_mut().find(|(name, _)| *name == child.name) {
                Some((_, count)) => {
                    *count += 1;
                    *count
                }
                None => {
                    seen.push((child.name.clone(), 0));
                    0
                }
            };

            let of = |name: &str| {
                child
                    .props
                    .iter()
                    .find(|prop| prop.name == name)
                    .map(|prop| prop.value.clone())
            };

            let key = match (of("UID"), of("RECURRENCE-ID")) {
                (Some(uid), Some(id)) => format!("{uid}/{id}"),
                (Some(uid), None) => uid,
                (None, _) => ordinal.to_string(),
            };

            out.push((child.name.clone(), key));
        }

        out
    }

    /// Render a calendar to wire bytes.
    pub fn render(comp: &Comp) -> Vec<u8> {
        let mut out = String::new();
        render_comp(comp, &mut out);
        out.into_bytes()
    }

    /// Render one component and everything nested in it.
    fn render_comp(comp: &Comp, out: &mut String) {
        out.push_str(&format!("BEGIN:{}\r\n", comp.name));

        for prop in &comp.props {
            out.push_str(&render_prop(prop));
        }

        for child in &comp.children {
            render_comp(child, out);
        }

        out.push_str(&format!("END:{}\r\n", comp.name));
    }

    /// Render one property line, folding it when it is marked folded, so the
    /// merge has a fold to preserve.
    pub fn render_prop(prop: &Prop) -> String {
        let mut line = prop.name.clone();

        for (name, value) in &prop.params {
            line.push_str(&format!(";{name}={value}"));
        }

        line.push(':');
        line.push_str(&prop.value);

        if !prop.folded || line.len() <= 45 {
            return format!("{line}\r\n");
        }

        let (head, tail) = line.split_at(45);
        format!("{head}\r\n {tail}\r\n")
    }

    /// A base calendar: a `VCALENDAR` holding a `VTIMEZONE`, one or two
    /// events, and, when an event recurs, an instance overriding it.
    fn base() -> impl Strategy<Value = Comp> {
        (
            any::<bool>(),
            1usize..=2,
            any::<bool>(),
            any::<bool>(),
            0usize..3,
            1usize..=2,
            1usize..=2,
        )
            .prop_map(
                |(timezone, events, recurring, override_one, start, attendees, alarms)| {
                    let mut children = Vec::new();

                    if timezone {
                        children.push(vtimezone());
                    }

                    for at in 0..events {
                        let uid = format!("event-{at}@example.com");
                        let recurs = recurring && at == 0;
                        children.push(vevent(&uid, recurs, start, attendees, alarms));

                        if recurs && override_one {
                            children.push(voverride(&uid, start));
                        }
                    }

                    Comp {
                        name: "VCALENDAR".to_owned(),
                        props: vec![
                            Prop::new("VERSION", "2.0"),
                            Prop::new("PRODID", "-//Pimalaya//ical-rs//EN"),
                        ],
                        children,
                    }
                },
            )
    }

    /// A `VTIMEZONE` with the two observances a side may redefine.
    fn vtimezone() -> Comp {
        Comp {
            name: "VTIMEZONE".to_owned(),
            props: vec![Prop::new("TZID", "Europe/Paris")],
            children: vec![
                Comp {
                    name: "STANDARD".to_owned(),
                    props: vec![
                        Prop::new("DTSTART", "19701025T030000"),
                        Prop::new("TZOFFSETFROM", "+0200"),
                        Prop::new("TZOFFSETTO", "+0100"),
                        Prop::new("RRULE", "FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10"),
                        Prop::new("TZNAME", "CET"),
                    ],
                    children: Vec::new(),
                },
                Comp {
                    name: "DAYLIGHT".to_owned(),
                    props: vec![
                        Prop::new("DTSTART", "19700329T020000"),
                        Prop::new("TZOFFSETFROM", "+0100"),
                        Prop::new("TZOFFSETTO", "+0200"),
                        Prop::new("RRULE", "FREQ=YEARLY;BYDAY=-1SU;BYMONTH=3"),
                        Prop::new("TZNAME", "CEST"),
                    ],
                    children: Vec::new(),
                },
            ],
        }
    }

    /// One event, its start floating, zoned or in UTC, optionally recurring,
    /// with an organiser, an attendee and an alarm.
    fn vevent(uid: &str, recurs: bool, start: usize, attendees: usize, alarms: usize) -> Comp {
        let dtstart = match start {
            0 => Prop::new("DTSTART", "20260105T090000"),
            1 => Prop::new("DTSTART", "20260105T090000Z"),
            _ => Prop::with("DTSTART", &[("TZID", "Europe/Paris")], "20260105T090000"),
        };

        let mut props = vec![
            Prop::new("UID", uid),
            Prop::new("DTSTAMP", "20260101T000000Z"),
            dtstart,
            Prop::new("SUMMARY", "Weekly sync"),
            Prop {
                folded: true,
                ..Prop::new(
                    "DESCRIPTION",
                    "A description long enough that the renderer folds it across two lines",
                )
            },
            Prop::new("LOCATION", "Room A"),
            Prop::new("CATEGORIES", "work,weekly"),
            Prop::new("TRANSP", "OPAQUE"),
            Prop::new("ORGANIZER", "mailto:chair@example.com"),
        ];

        // NOTE: More than one attendee is what makes the positional pairing
        // of same-named properties observable: remove one and every later
        // one is renumbered.
        for who in ["ada", "zoe"].iter().take(attendees) {
            props.push(Prop::with(
                "ATTENDEE",
                &[("PARTSTAT", "NEEDS-ACTION"), ("CN", who)],
                &format!("mailto:{who}@example.com"),
            ));
        }

        if recurs {
            props.push(Prop::new("RRULE", "FREQ=WEEKLY;COUNT=5"));
            props.push(Prop::new("EXDATE", "20260119T090000Z,20260126T090000Z"));
        }

        let children = [("DISPLAY", "-PT10M"), ("AUDIO", "-PT20M")]
            .iter()
            .take(alarms)
            .map(|(action, trigger)| Comp {
                name: "VALARM".to_owned(),
                props: vec![
                    Prop::new("ACTION", action),
                    Prop::new("TRIGGER", trigger),
                    Prop::new("DESCRIPTION", "Reminder"),
                ],
                children: Vec::new(),
            })
            .collect();

        Comp {
            name: "VEVENT".to_owned(),
            props,
            children,
        }
    }

    /// An instance overriding one occurrence of a series, addressed by its
    /// `RECURRENCE-ID` rather than by its position.
    fn voverride(uid: &str, start: usize) -> Comp {
        let dtstart = match start {
            1 => "20260112T100000Z",
            _ => "20260112T100000",
        };

        Comp {
            name: "VEVENT".to_owned(),
            props: vec![
                Prop::new("UID", uid),
                Prop::new("DTSTAMP", "20260101T000000Z"),
                Prop::new("RECURRENCE-ID", "20260112T090000Z"),
                Prop::new("DTSTART", dtstart),
                Prop::new("SUMMARY", "Weekly sync, moved"),
                Prop::new("ORGANIZER", "mailto:chair@example.com"),
            ],
            children: Vec::new(),
        }
    }

    /// One edit a side makes on its own.
    fn edit() -> impl Strategy<Value = Edit> {
        let slot = 0usize..64;
        let comp = 0usize..8;
        let seed = 0usize..4;

        prop_oneof![
            5 => (slot.clone(), seed.clone()).prop_map(|(slot, seed)| Edit::SetValue { slot, seed }),
            2 => slot.clone().prop_map(|slot| Edit::RemoveProp { slot }),
            2 => (comp.clone(), seed.clone()).prop_map(|(comp, seed)| Edit::AddProp { comp, seed }),
            2 => (slot.clone(), seed.clone()).prop_map(|(slot, seed)| Edit::AddListItem { slot, seed }),
            1 => slot.clone().prop_map(|slot| Edit::RemoveListItem { slot }),
            2 => (slot.clone(), seed.clone()).prop_map(|(slot, seed)| Edit::SetParam { slot, seed }),
            1 => slot.clone().prop_map(|slot| Edit::RemoveParam { slot }),
            1 => (slot, seed.clone()).prop_map(|(slot, seed)| Edit::AddParam { slot, seed }),
            1 => (comp.clone(), seed).prop_map(|(comp, seed)| Edit::AddAlarm { comp, seed }),
            1 => comp.prop_map(|comp| Edit::RemoveComp { comp }),
        ]
    }

    /// A pair of edits, one per side, aimed at one target.
    ///
    /// This is what makes the suite worth running: two edits drawn
    /// independently would almost never land on the same field, and every
    /// collision rule would go untested. The pairs cover both sides writing a
    /// value, an update meeting a removal in both directions, two parameters,
    /// two list items, two additions of one name, and a component removed
    /// under an edit.
    fn shared() -> impl Strategy<Value = (Vec<Edit>, Vec<Edit>)> {
        let slot = 0usize..64;
        let comp = 0usize..8;

        prop_oneof![
            // NOTE: One side removes a neighbour of the property both sides
            // contest. Properties are paired by position once equality fails,
            // so removing an earlier same-named sibling renumbers the one
            // both sides wrote to and the two actions stop meeting.
            3 => (slot.clone(), slot.clone()).prop_map(|(one, two)| (
                vec![
                    Edit::RemoveProp { slot: one },
                    Edit::SetParam { slot: two, seed: 1 },
                ],
                vec![Edit::SetParam { slot: two, seed: 2 }],
            )),
            3 => (slot.clone(), slot.clone()).prop_map(|(one, two)| (
                vec![
                    Edit::RemoveProp { slot: one },
                    Edit::SetValue { slot: two, seed: 1 },
                ],
                vec![Edit::SetValue { slot: two, seed: 2 }],
            )),
            3 => (comp.clone(), slot.clone()).prop_map(|(one, two)| (
                vec![
                    Edit::RemoveComp { comp: one },
                    Edit::SetValue { slot: two, seed: 1 },
                ],
                vec![Edit::SetValue { slot: two, seed: 2 }],
            )),
            10 => slot.clone().prop_map(|slot| (vec![Edit::SetValue { slot, seed: 1 }], vec![Edit::SetValue { slot, seed: 2 }])),
            3 => slot.clone().prop_map(|slot| (vec![Edit::SetValue { slot, seed: 1 }], vec![Edit::RemoveProp { slot }])),
            3 => slot.clone().prop_map(|slot| (vec![Edit::RemoveProp { slot }], vec![Edit::SetValue { slot, seed: 2 }])),
            3 => slot.clone().prop_map(|slot| (vec![Edit::SetParam { slot, seed: 1 }], vec![Edit::SetParam { slot, seed: 2 }])),
            2 => slot.clone().prop_map(|slot| (vec![Edit::SetParam { slot, seed: 1 }], vec![Edit::RemoveParam { slot }])),
            2 => slot.clone().prop_map(|slot| (vec![Edit::AddParam { slot, seed: 1 }], vec![Edit::AddParam { slot, seed: 2 }])),
            3 => slot.clone().prop_map(|slot| (vec![Edit::AddListItem { slot, seed: 1 }], vec![Edit::AddListItem { slot, seed: 2 }])),
            2 => slot.clone().prop_map(|slot| (vec![Edit::AddListItem { slot, seed: 1 }], vec![Edit::RemoveProp { slot }])),
            2 => slot.clone().prop_map(|slot| (vec![Edit::RemoveProp { slot }], vec![Edit::AddListItem { slot, seed: 2 }])),
            2 => comp.clone().prop_map(|comp| (vec![Edit::AddProp { comp, seed: 1 }], vec![Edit::AddProp { comp, seed: 2 }])),
            2 => comp.clone().prop_map(|comp| (vec![Edit::AddAlarm { comp, seed: 0 }], vec![Edit::AddAlarm { comp, seed: 1 }])),
            2 => (comp.clone(), slot).prop_map(|(comp, slot)| (vec![Edit::RemoveComp { comp }], vec![Edit::SetValue { slot, seed: 2 }])),
            2 => comp.prop_map(|comp| (vec![Edit::AddAlarm { comp, seed: 1 }], vec![Edit::RemoveComp { comp }])),
        ]
    }

    /// A base calendar and two edits of it, most of them contesting the same
    /// fields.
    pub fn scenario() -> impl Strategy<Value = Scenario> {
        (
            base(),
            prop::collection::vec(shared(), 1..3),
            prop::collection::vec(edit(), 0..2),
            prop::collection::vec(edit(), 0..2),
        )
            .prop_map(|(base, shared, left_only, right_only)| {
                let mut left: Vec<Edit> =
                    shared.iter().flat_map(|(left, _)| left.clone()).collect();
                let mut right: Vec<Edit> =
                    shared.iter().flat_map(|(_, right)| right.clone()).collect();

                left.extend(left_only);
                right.extend(right_only);

                Scenario { base, left, right }
            })
    }
}

/// A group of same-named properties of one component, or of same-named sibling
/// components of one parent, whose members a positional index tells apart.
type GroupKey = (Vec<(String, String)>, String);

/// How many members each group holds.
fn group_counts(model: &IcalModel) -> BTreeMap<GroupKey, usize> {
    let mut out: BTreeMap<GroupKey, usize> = BTreeMap::new();

    for key in model.keys() {
        match &key.slot {
            FieldSlot::Prop => {
                *out.entry((key.component.clone(), key.prop.clone()))
                    .or_default() += 1;
            }
            FieldSlot::Component if !key.component.is_empty() => {
                let (parent, last) = key.component.split_at(key.component.len() - 1);
                *out.entry((parent.to_vec(), last[0].0.clone())).or_default() += 1;
            }
            _ => {}
        }
    }

    out
}

/// The groups a position does not tell apart, so no law can be stated over
/// their fields by that position.
///
/// A group whose members iCalendar identifies by what they name is safe
/// whatever happens to it, since the model keys them the way the merge does. Of
/// the rest, a group nobody took a member out of is safe: additions land at the
/// end and renumber nothing. A group of at most one member that nobody added to
/// is safe too: index zero is the only index it can have, and it either
/// survives or it does not. Anything else mixes a removal with an addition, and
/// index `n` then names a different member in each calendar that saw it.
fn unstable_groups(merged: &Merged<'_>) -> BTreeSet<GroupKey> {
    let base = group_counts(&merged.base);
    let sides = [
        group_counts(&merged.left),
        group_counts(&merged.right),
        group_counts(&merged.merged),
    ];

    let mut groups: BTreeSet<GroupKey> = base.keys().cloned().collect();

    for held in &sides {
        groups.extend(held.keys().cloned());
    }

    let positional = positional_groups(merged);

    groups
        .into_iter()
        .filter(|group| {
            if !positional.contains(group) {
                return false;
            }

            let held = base.get(group).copied().unwrap_or(0);
            let counts: Vec<usize> = sides
                .iter()
                .map(|side| side.get(group).copied().unwrap_or(0))
                .collect();

            let nothing_removed = counts.iter().all(|count| *count >= held);
            let nothing_added = held <= 1 && counts.iter().all(|count| *count <= held);

            !nothing_removed && !nothing_added
        })
        .collect()
}

/// The groups whose members a position tells apart, as opposed to the ones
/// iCalendar gives an identity of their own.
fn positional_groups(merged: &Merged<'_>) -> BTreeSet<GroupKey> {
    let mut out = BTreeSet::new();

    for model in [&merged.base, &merged.left, &merged.right, &merged.merged] {
        for (group, members) in group_members(model) {
            if members.keys().all(|at| at.starts_with('#') || ordinal(at)) {
                out.insert(group);
            }
        }
    }

    out
}

/// Whether a member key is a position among same-named siblings rather than an
/// identity of its own.
fn ordinal(at: &str) -> bool {
    !at.is_empty() && at.bytes().all(|held| held.is_ascii_digit())
}

/// The shifted groups whose members are sibling components.
///
/// A component carrying no `UID` is matched by its position, and iCalendar
/// gives it nothing else to be matched by, so a side that removed one of them
/// pairs the survivors with the base by position: the base's first alarm is
/// matched with what is now the first alarm, and the merge reports a change
/// nobody made. Every action still lands or is reported, which is what
/// [`every_change_lands_or_is_reported`] holds it to, but the bytes of the
/// lines involved are not the bytes anybody wrote.
fn shifted_component_groups(merged: &Merged<'_>) -> BTreeSet<GroupKey> {
    let members = group_members(&merged.base);

    shifted_groups(merged)
        .into_iter()
        .filter(|group| {
            members
                .get(group)
                .is_some_and(|held| held.keys().all(|at| ordinal(at)))
        })
        .collect()
}

/// The identity of each member of each group, in the order a positional index
/// would name them.
///
/// A property's identity is its value, or the set of its list items when it has
/// no single value. A component's is the canonical text of its own properties.
/// Only components matched by position get an entry, since one matched by `UID`
/// cannot be renumbered by a neighbour's removal.
fn group_members(model: &IcalModel) -> BTreeMap<GroupKey, BTreeMap<String, String>> {
    let mut out: BTreeMap<GroupKey, BTreeMap<String, Vec<String>>> = BTreeMap::new();

    for (key, value) in model {
        // NOTE: The identity is the value alone. A parameter one side edited
        // does not make a property a different property, and counting it in
        // would hide a renumbering behind an ordinary edit.
        if !matches!(key.slot, FieldSlot::Value | FieldSlot::Item(_)) {
            continue;
        }

        out.entry((key.component.clone(), key.prop.clone()))
            .or_default()
            .entry(key.at.clone())
            .or_default()
            .push(format!("{:?}={value}", key.slot));

        let Some((last, parent)) = key.component.split_last() else {
            continue;
        };

        if !last.1.bytes().all(|held| held.is_ascii_digit()) {
            continue;
        }

        out.entry((parent.to_vec(), last.0.clone()))
            .or_default()
            .entry(last.1.clone())
            .or_default()
            .push(format!("{}/{}/{:?}={value}", key.prop, key.at, key.slot));
    }

    out.into_iter()
        .map(|(group, members)| {
            let members = members
                .into_iter()
                .map(|(at, mut fields)| {
                    fields.sort();
                    (at, fields.join(" "))
                })
                .collect();

            (group, members)
        })
        .collect()
}

/// The groups a side renumbered, among the ones a position tells apart.
///
/// This is a limit of the model rather than of the merge. A field of the model
/// is keyed by the position its property holds in the calendar it was read
/// from, so where a side dropped a member of a group the same key names one
/// property in the base and its neighbour in that side, and comparing the four
/// models field by field would compare two different properties. The merge
/// itself translates a base position through the baseline side's own removals
/// before resolving it, which is what
/// [`a_removal_does_not_duplicate_a_neighbour`] pins; the weaker
/// [`completeness_of_records`] is what speaks about these groups.
///
/// A group whose members iCalendar identifies by what they name is never in
/// here: the model keys those by the identity, exactly as the merge does.
///
/// A position whose member merely changed is not renumbered: the test is that
/// the side's member at that position is one the base held somewhere else.
fn shifted_groups(merged: &Merged<'_>) -> BTreeSet<GroupKey> {
    let base = group_members(&merged.base);
    let positional = positional_groups(merged);

    let renumbered = |side: &IcalModel| {
        let held = group_members(side);

        base.iter()
            .filter(|(group, members)| {
                if !positional.contains(*group) {
                    return false;
                }

                let Some(side) = held.get(*group) else {
                    return false;
                };

                // A group of one member cannot be renumbered: index zero is
                // the only index it has. A larger one is renumbered as soon
                // as a member joins or leaves it, since the merge addresses
                // each member by the position it held in the base.
                if members.len() > 1 && side.len() != members.len() {
                    return true;
                }

                members.iter().any(|(at, member)| {
                    side.get(at).is_some_and(|side| {
                        side != member && members.values().any(|other| other == side)
                    })
                })
            })
            .map(|(group, _)| group.clone())
            .collect::<BTreeSet<_>>()
    };

    let touched = |side: &IcalModel| {
        let mut out: BTreeSet<GroupKey> = BTreeSet::new();

        for key in universe(merged) {
            if merged.base.get(&key) == side.get(&key) {
                continue;
            }

            out.insert((key.component.clone(), key.prop.clone()));

            for depth in 0..key.component.len() {
                out.insert((
                    key.component[..depth].to_vec(),
                    key.component[depth].0.clone(),
                ));
            }
        }

        out
    };

    // NOTE: A right side that renumbered a group is unsafe on its own,
    // since its own actions are the ones addressed by a base position. A left
    // side that renumbered one is unsafe only where the right side has
    // something to replay into it.
    let mut out = renumbered(&merged.right);

    out.extend(
        renumbered(&merged.left)
            .intersection(&touched(&merged.right))
            .cloned(),
    );

    out
}

/// What the completeness law needs to know about a merge, gathered once.
///
/// It asks the same question about every property address, and asking it by
/// sweeping the four models per field turns a law into a quadratic scan of a
/// calendar. The answers are collected here instead.
struct Shape {
    /// Which of the base, the left side and the right side holds each
    /// property.
    present: BTreeMap<model::Address, (bool, bool, bool)>,
}

impl Shape {
    /// Gather it.
    fn of(merged: &Merged<'_>) -> Self {
        let mut present: BTreeMap<model::Address, (bool, bool, bool)> = BTreeMap::new();

        for (at, model) in [&merged.base, &merged.left, &merged.right]
            .into_iter()
            .enumerate()
        {
            for key in model.keys() {
                if key.slot != FieldSlot::Prop {
                    continue;
                }

                let address = (key.component.clone(), key.prop.clone(), key.at.clone());
                let held = present.entry(address).or_default();

                match at {
                    0 => held.0 = true,
                    1 => held.1 = true,
                    _ => held.2 = true,
                }
            }
        }

        Self { present }
    }

    /// Whether one side removed this property outright.
    fn removed_outright(&self, address: &model::Address) -> bool {
        match self.present.get(address) {
            Some((true, left, right)) => !left || !right,
            _ => false,
        }
    }
}

/// Whether a field belongs to one of these groups, or sits under a component
/// one of them holds.
fn in_groups(key: &FieldKey, groups: &BTreeSet<GroupKey>) -> bool {
    if groups.contains(&(key.component.clone(), key.prop.clone())) {
        return true;
    }

    groups.iter().any(|(parent, name)| {
        key.component.len() > parent.len()
            && key.component.starts_with(parent)
            && key.component[parent.len()].0 == *name
    })
}

/// Every field named by any of the four calendars.
fn universe(merged: &Merged<'_>) -> BTreeSet<FieldKey> {
    let mut out = BTreeSet::new();

    for model in [&merged.base, &merged.left, &merged.right, &merged.merged] {
        out.extend(model.keys().cloned());
    }

    out
}

/// The completeness law: every change either lands or is reported.
///
/// For every field of the merged calendar, it equals what one side made it and
/// that side changed it, or it equals the base and neither side changed it. A
/// change that did not land is named in the report. Nothing is silently
/// dropped, and nothing appears that neither side wrote.
///
/// One subset is left out, because the law cannot be stated over it rather
/// than because it holds. The fields of a group the model addresses by a
/// position a side renumbered are handled by the weaker
/// [`completeness_of_records`], which does not need a position to be an
/// identity.
fn completeness(merged: &Merged<'_>) -> Result<(), String> {
    let universe = universe(merged);
    let contested = model::contested(&merged.report, &universe);
    let unstable = unstable_groups(merged);
    let shifted = shifted_groups(merged);
    let shape = Shape::of(merged);

    for key in &universe {
        if in_groups(key, &unstable) || in_groups(key, &shifted) {
            continue;
        }

        let address = (key.component.clone(), key.prop.clone(), key.at.clone());

        let b = merged.base.get(key);
        let l = merged.left.get(key);
        let r = merged.right.get(key);
        let m = merged.merged.get(key);

        let named = format!("{key:?}\n  base {b:?}\n  left {l:?}\n  right {r:?}\n  merged {m:?}");
        let address_reported = contested.iter().any(|held| {
            held.component == key.component && held.prop == key.prop && held.at == key.at
        });

        // NOTE: A reported collision about a property covers the property's
        // existence, which the report never names on its own, and covers every
        // field of a property one side removed outright, since a removal and
        // an update cannot both be honoured field by field: the surviving
        // line is the updating side's, parameters and all.
        let reported = contested.contains(key)
            || (address_reported
                && (key.slot == FieldSlot::Prop || shape.removed_outright(&address)));

        if m != b && m != l && m != r {
            return Err(format!("a field neither side wrote appeared:\n{named}"));
        }

        match (l != b, r != b) {
            (false, false) => {
                if m != b {
                    return Err(format!("a field nobody changed moved:\n{named}"));
                }
            }
            (true, false) => {
                if m != l && !reported {
                    return Err(format!("a left-side change vanished unreported:\n{named}"));
                }
            }
            (false, true) => {
                if m != r && !reported {
                    return Err(format!("a right-side change vanished unreported:\n{named}"));
                }
            }
            (true, true) if l == r => {
                if m != l && !reported {
                    return Err(format!(
                        "a change both sides agreed on vanished unreported:\n{named}"
                    ));
                }
            }
            (true, true) => {
                if m != l && m != r {
                    return Err(format!("both sides' changes vanished:\n{named}"));
                }

                if !reported {
                    return Err(format!("a collision went unreported:\n{named}"));
                }
            }
        }
    }

    Ok(())
}

/// The weaker law for the groups a positional index does not tell apart.
///
/// It gives up on saying which member is which and asserts only what does not
/// need that: no member of the merged calendar was invented, no member a side
/// added went missing without a word, and no member both sides removed came
/// back.
fn completeness_of_records(merged: &Merged<'_>) -> Result<(), String> {
    let universe = universe(merged);
    let contested = model::contested(&merged.report, &universe);
    let unstable = unstable_groups(merged);
    let shifted = shifted_groups(merged);

    let base = records(&merged.base);
    let left = records(&merged.left);
    let right = records(&merged.right);
    let held = records(&merged.merged);

    let mut groups: BTreeSet<&GroupKey> = BTreeSet::new();
    groups.extend(base.keys());
    groups.extend(left.keys());
    groups.extend(right.keys());
    groups.extend(held.keys());

    for group in groups {
        if !unstable.contains(group) || shifted.contains(group) {
            continue;
        }

        let empty = Vec::new();
        let b = base.get(group).unwrap_or(&empty);
        let l = left.get(group).unwrap_or(&empty);
        let r = right.get(group).unwrap_or(&empty);
        let m = held.get(group).unwrap_or(&empty);

        let reported = contested
            .iter()
            .any(|key| (key.component.clone(), key.prop.clone()) == *group);

        for record in m {
            if !b.contains(record) && !l.contains(record) && !r.contains(record) {
                return Err(format!(
                    "a property neither side wrote appeared in {group:?}: {record}\n  \
                     base {b:?}\n  left {l:?}\n  right {r:?}\n  merged {m:?}"
                ));
            }
        }

        for (side, records) in [("left", l), ("right", r)] {
            for record in records {
                if b.contains(record) || m.contains(record) || reported {
                    continue;
                }

                return Err(format!(
                    "a {side}-side property vanished unreported from {group:?}: {record}"
                ));
            }
        }

        for record in b {
            if l.contains(record) || r.contains(record) || !m.contains(record) || reported {
                continue;
            }

            return Err(format!(
                "a property both sides removed came back in {group:?}: {record}"
            ));
        }
    }

    Ok(())
}

/// The properties of a calendar, grouped by component and name, each rendered
/// as one canonical record so two occurrences can be told apart without their
/// position.
fn records(model: &IcalModel) -> BTreeMap<GroupKey, Vec<String>> {
    let mut out: BTreeMap<GroupKey, BTreeMap<String, Vec<String>>> = BTreeMap::new();

    for (key, value) in model {
        if key.slot == FieldSlot::Component || key.slot == FieldSlot::Prop {
            continue;
        }

        out.entry((key.component.clone(), key.prop.clone()))
            .or_default()
            .entry(key.at.clone())
            .or_default()
            .push(format!("{:?}={value}", key.slot));
    }

    out.into_iter()
        .map(|(group, occurrences)| {
            let records = occurrences
                .into_values()
                .map(|mut fields| {
                    fields.sort();
                    fields.join(" ")
                })
                .collect();

            (group, records)
        })
        .collect()
}

/// Whether a scenario is one the field-level reference can speak about.
///
/// Three shapes are excluded, each because the reference cannot express what
/// the real merge does rather than because either is wrong.
///
/// A group a positional index does not tell apart is excluded, since the
/// reference matches properties by that index and the merge matches them by
/// content first (see [`unstable_groups`]).
///
/// A component one side removed while the other changed something inside it,
/// at any depth, is excluded: the merge collapses every such change into one
/// collision against the removal, and which of the removed component's own
/// actions that collision lands on depends on the order the diff emitted them,
/// which a field-level reference has no way to model.
fn comparable(merged: &Merged<'_>) -> bool {
    if !unstable_groups(merged).is_empty() || !shifted_groups(merged).is_empty() {
        return false;
    }

    let paths = |model: &IcalModel| {
        model
            .keys()
            .filter(|key| key.slot == FieldSlot::Component)
            .map(|key| key.component.clone())
            .collect::<BTreeSet<_>>()
    };

    let left = paths(&merged.left);
    let right = paths(&merged.right);
    let universe = universe(merged);

    for path in paths(&merged.base) {
        let gone_left = !left.contains(&path);
        let gone_right = !right.contains(&path);

        if !gone_left && !gone_right {
            continue;
        }

        let touched = |model: &IcalModel| {
            universe.iter().any(|key| {
                key.component.starts_with(&path) && merged.base.get(key) != model.get(key)
            })
        };

        if gone_left && touched(&merged.right) {
            return false;
        }

        if gone_right && touched(&merged.left) {
            return false;
        }
    }

    true
}

/// Whether the real merge reports exactly the collisions the reference does.
///
/// Both name a collision where the two sides disagreed and neither names one
/// where they agreed, so a surplus on either side is a disagreement about what
/// a collision is rather than about how it is spelt.
fn agrees_on_conflicts(
    merged: &Merged<'_>,
    reference: &BTreeSet<model::Address>,
) -> Result<(), String> {
    let held = model::contested_addresses(&merged.report);

    for address in reference {
        if !held.contains(address) {
            return Err(format!(
                "a collision the reference names went unreported: {address:?}"
            ));
        }
    }

    for conflict in &merged.report.conflicts {
        if matches!(
            conflict.left,
            ical::tree::merge::IcalMergeReason::Recurrence(_)
        ) {
            continue;
        }

        let address = model::address_of(&conflict.right);

        if !reference.contains(&address) {
            return Err(format!(
                "a collision was reported that the two sides did not have: {address:?}, \
                 reported as {:?}",
                conflict.left
            ));
        }
    }

    Ok(())
}

/// The first field two projections disagree on, if any.
///
/// A whole-model equality assertion prints two calendars, which nobody can
/// read; this names the one field that differs.
fn differs(held: &IcalModel, wanted: &IcalModel) -> Option<String> {
    let mut keys: BTreeSet<&FieldKey> = BTreeSet::new();
    keys.extend(held.keys());
    keys.extend(wanted.keys());

    keys.into_iter().find_map(|key| {
        let (one, two) = (held.get(key), wanted.get(key));

        (one != two).then(|| format!("{key:?}\n  held {one:?}\n  wanted {two:?}"))
    })
}

proptest! {
    #![proptest_config(Config { cases: cases(), ..Config::default() })]

    /// Merging a side with itself yields that side, byte for byte.
    #[test]
    fn merging_a_side_with_itself_yields_it(scenario in scenario()) {
        let base = scenario.base_bytes();
        let left = scenario.left_bytes();

        let Some(merged) = run(&base, &left, &left) else {
            return Ok(());
        };

        prop_assert_eq!(bytes(&merged.report), left.clone());
    }

    /// A side that changed nothing yields the other side, and a base neither
    /// side changed yields the base.
    #[test]
    fn an_unchanged_side_yields_the_other(scenario in scenario()) {
        let base = scenario.base_bytes();
        let left = scenario.left_bytes();
        let right = scenario.right_bytes();

        let Some(only_right) = run(&base, &base, &right) else {
            return Ok(());
        };
        let Some(only_left) = merge(&base, &left, &base) else {
            return Ok(());
        };
        let Some(neither) = merge(&base, &base, &base) else {
            return Ok(());
        };

        if let Some(why) = differs(&only_right.merged, &only_right.right) {
            return Err(TestCaseError::fail(format!(
                "a side that changed nothing did not yield the other:\n{why}"
            )));
        }
        prop_assert!(only_right.report.conflicts.is_empty());

        prop_assert_eq!(bytes(&only_left), left.clone());
        prop_assert!(only_left.conflicts.is_empty());

        prop_assert_eq!(bytes(&neither), base.clone());
        prop_assert!(neither.conflicts.is_empty());
    }

    /// Swapping the two sides reports the same set of collided properties,
    /// modulo which side each action came from.
    #[test]
    fn conflict_reporting_is_symmetric(scenario in scenario()) {
        let base = scenario.base_bytes();
        let left = scenario.left_bytes();
        let right = scenario.right_bytes();

        let Some(forward) = run(&base, &left, &right) else {
            return Ok(());
        };
        let Some(backward) = run(&base, &right, &left) else {
            return Ok(());
        };

        if !comparable(&forward) || !comparable(&backward) {
            return Ok(());
        }

        prop_assert_eq!(
            model::contested_addresses(&forward.report),
            model::contested_addresses(&backward.report)
        );
    }

    /// The merged calendar always parses again, to the same bytes.
    #[test]
    fn the_merged_calendar_reparses(scenario in scenario()) {
        let base = scenario.base_bytes();
        let left = scenario.left_bytes();
        let right = scenario.right_bytes();

        let Some(report) = merge(&base, &left, &right) else {
            return Ok(());
        };

        let merged = bytes(&report);
        let reparsed = IcalCst::parse(&merged);

        prop_assert!(
            reparsed.is_ok(),
            "the merged calendar does not parse: {}",
            String::from_utf8_lossy(&merged)
        );
        prop_assert_eq!(reparsed.unwrap().to_bytes(), merged);
    }

    /// A line neither side touched comes out byte for byte, folds included.
    #[test]
    fn an_untouched_line_keeps_its_bytes(scenario in scenario()) {
        let base = scenario.base_bytes();
        let left = scenario.left_bytes();
        let right = scenario.right_bytes();
        let untouched = scenario.untouched_lines();

        let Some(held) = run(&base, &left, &right) else {
            return Ok(());
        };

        // NOTE: A side that removed one of a group of sibling components
        // iCalendar gives no identity to renumbers the survivors, and the
        // merge then pairs the base's first with the side's first. See
        // shifted_component_groups.
        if !shifted_component_groups(&held).is_empty() {
            return Ok(());
        }

        let merged = String::from_utf8(bytes(&held.report)).expect("valid UTF-8");

        for line in &untouched {
            prop_assert!(
                merged.contains(line.as_str()),
                "an untouched line lost its bytes: {line:?}\nleft:\n{}\nright:\n{}\nmerged:\n{merged}",
                String::from_utf8_lossy(&left),
                String::from_utf8_lossy(&right)
            );
        }
    }

    /// Where both sides wrote a different value into one field, the merged
    /// calendar carries the left side's: the left side is git's `ours`.
    #[test]
    fn a_collision_carries_the_left_value(scenario in scenario()) {
        let base = scenario.base_bytes();
        let left = scenario.left_bytes();
        let right = scenario.right_bytes();

        let Some(merged) = run(&base, &left, &right) else {
            return Ok(());
        };

        if !comparable(&merged) {
            return Ok(());
        }

        for key in universe(&merged) {
            let b = merged.base.get(&key);
            let l = merged.left.get(&key);
            let r = merged.right.get(&key);

            if !(l != b && r != b && l != r && l.is_some() && r.is_some()) {
                continue;
            }

            prop_assert_eq!(merged.merged.get(&key), l, "the left value did not win");
        }
    }

    /// The completeness law.
    #[test]
    fn every_change_lands_or_is_reported(scenario in scenario()) {
        let base = scenario.base_bytes();
        let left = scenario.left_bytes();
        let right = scenario.right_bytes();

        let Some(merged) = run(&base, &left, &right) else {
            return Ok(());
        };

        if let Err(why) = completeness(&merged) {
            return Err(TestCaseError::fail(why));
        }

        if let Err(why) = completeness_of_records(&merged) {
            return Err(TestCaseError::fail(why));
        }
    }

    /// The real merge and the naive reference agree on the merged content and
    /// on the set of contested properties.
    #[test]
    fn the_reference_merge_agrees(scenario in scenario()) {
        let base = scenario.base_bytes();
        let left = scenario.left_bytes();
        let right = scenario.right_bytes();

        let Some(merged) = run(&base, &left, &right) else {
            return Ok(());
        };

        if !comparable(&merged) {
            return Ok(());
        }

        let reference = reference::merge(&merged.base, &merged.left, &merged.right);

        if let Some(why) = differs(&merged.merged, &reference.merged) {
            return Err(TestCaseError::fail(format!(
                "the merged content differs from the reference:\n{why}"
            )));
        }
        if let Err(why) = agrees_on_conflicts(&merged, &reference.contested) {
            return Err(TestCaseError::fail(why));
        }
    }
}

/// The share of generated scenarios that actually collide, and the share the
/// laws can speak about.
///
/// A generator that rarely puts both sides on one field would make every law
/// pass without testing anything, so the rate is measured rather than assumed,
/// and the floor fails the suite when the generator drifts.
#[test]
fn the_generator_collides_often_enough() {
    let mut runner = TestRunner::new(Config {
        cases: 2048,
        ..Config::default()
    });
    let strategy = scenario();

    let mut total = 0usize;
    let mut collided = 0usize;
    let mut reported = 0usize;
    let mut compared = 0usize;

    for _ in 0..2048 {
        let case = strategy
            .new_tree(&mut runner)
            .expect("a generated scenario")
            .current();

        let base = case.base_bytes();
        let left = case.left_bytes();
        let right = case.right_bytes();

        let Some(merged) = run(&base, &left, &right) else {
            continue;
        };

        total += 1;

        if !merged.report.conflicts.is_empty() {
            reported += 1;
        }

        if merged.report.conflicts.iter().any(|conflict| {
            matches!(
                conflict.left,
                ical::tree::merge::IcalMergeReason::Divergent(_)
            )
        }) {
            collided += 1;
        }

        if comparable(&merged) {
            compared += 1;
        }
    }

    let rate = collided as f64 / total as f64;
    let comparable = compared as f64 / total as f64;

    println!(
        "collisions {collided}/{total} = {rate:.3}, \
         any conflict {reported}/{total}, \
         comparable to the reference {compared}/{total} = {comparable:.3}"
    );

    assert!(
        rate > 0.5,
        "the generator stopped putting both sides on one field: {rate:.3}"
    );
    assert!(
        comparable > 0.2,
        "the reference no longer sees enough scenarios: {comparable:.3}"
    );
}

/// The laws, replayed over the frozen corpora rather than over generated
/// calendars.
///
/// The generator builds calendars this crate's author would write. The corpora
/// hold calendars three decades of vendors wrote, folded in their own ways,
/// carrying properties and components the generator never emits. Each fixture
/// becomes a base, two sides are synthesized from it by rewriting the value of
/// a few of its own lines (one of them the same line on both sides, so the
/// merge has something to collide), and the same laws are stated over the
/// result.
mod corpus {
    use std::{fs, path::PathBuf};

    /// The corpora that hold whole calendars. The recur corpus holds rule
    /// expansions rather than calendars, so nothing here can be seeded from it.
    pub const CORPORA: [&str; 5] = ["libical", "ical4j", "icaljs", "rfc", "vcalendar"];

    /// Every fixture of every corpus, as a name and its raw bytes.
    pub fn fixtures() -> Vec<(String, Vec<u8>)> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
        let mut out = Vec::new();

        for corpus in CORPORA {
            let dir = root.join(corpus);

            for entry in fs::read_dir(&dir).expect("a readable corpus") {
                let path = entry.expect("a corpus entry").path();

                if path.extension().and_then(|held| held.to_str()) != Some("ics") {
                    continue;
                }

                let name = format!(
                    "{corpus}/{}",
                    path.file_name().and_then(|held| held.to_str()).unwrap()
                );

                out.push((name, fs::read(&path).expect("a readable fixture")));
            }
        }

        out.sort_by(|(one, _), (two, _)| one.cmp(two));
        out
    }

    /// One logical line of a fixture: the byte range it occupies, folds
    /// included, and where its value starts.
    pub struct Block {
        /// Where the line starts in the fixture.
        pub start: usize,
        /// Where it ends, its ending included.
        pub end: usize,
        /// Where its value starts, or `None` when it has no colon, is folded,
        /// or names something an edit must not touch.
        pub value: Option<usize>,
    }

    /// The logical lines of a fixture, in order.
    ///
    /// A line is editable only when it is unfolded, holds a colon, sits inside
    /// the first calendar (which is all a parse keeps) and names something
    /// other than the envelope or the identity a component is matched by.
    pub fn blocks(input: &[u8]) -> Vec<Block> {
        let mut out: Vec<Block> = Vec::new();
        let mut at = 0;
        let mut ended = false;

        while at < input.len() {
            let end = match input[at..].iter().position(|held| *held == b'\n') {
                Some(held) => at + held + 1,
                None => input.len(),
            };

            let folded = input[at] == b' ' || input[at] == b'\t';

            if folded && let Some(last) = out.last_mut() {
                last.end = end;
                last.value = None;
                at = end;
                continue;
            }

            let colon = input[at..end].iter().position(|held| *held == b':');
            let name = colon
                .map(|held| String::from_utf8_lossy(&input[at..at + held]).to_ascii_uppercase())
                .unwrap_or_default();

            let value = match colon {
                Some(colon)
                    if !ended
                        && !name.is_empty()
                        && !name.contains(';')
                        && !matches!(
                            name.as_str(),
                            "BEGIN" | "END" | "UID" | "RECURRENCE-ID" | "VERSION"
                        ) =>
                {
                    Some(at + colon + 1)
                }
                _ => None,
            };

            if name == "END"
                && input[at..end]
                    .to_ascii_uppercase()
                    .windows(9)
                    .any(|held| held == b"VCALENDAR")
            {
                ended = true;
            }

            out.push(Block {
                start: at,
                end,
                value,
            });
            at = end;
        }

        out
    }

    /// A fixture with a suffix appended to the value of the chosen lines, which
    /// is an edit every value type survives syntactically.
    pub fn edited(input: &[u8], blocks: &[Block], chosen: &[usize], suffix: &str) -> Vec<u8> {
        let mut out = Vec::new();
        let mut at = 0;

        for held in chosen {
            let Some(block) = blocks.get(*held) else {
                continue;
            };
            let Some(value) = block.value else {
                continue;
            };

            let ending = block.end - trailing(&input[block.start..block.end]);

            out.extend_from_slice(&input[at..ending]);
            out.extend_from_slice(suffix.as_bytes());
            at = ending;

            let _ = value;
        }

        out.extend_from_slice(&input[at..]);
        out
    }

    /// How many bytes of line ending one block carries.
    fn trailing(block: &[u8]) -> usize {
        match block {
            [.., b'\r', b'\n'] => 2,
            [.., b'\n'] => 1,
            _ => 0,
        }
    }

    /// The editable lines of a fixture, in order.
    pub fn editable(blocks: &[Block]) -> Vec<usize> {
        blocks
            .iter()
            .enumerate()
            .filter(|(_, block)| block.value.is_some())
            .map(|(at, _)| at)
            .collect()
    }

    /// A small deterministic spread, so every fixture picks different lines
    /// without a random source the run cannot reproduce.
    pub fn spread(name: &str, round: usize) -> usize {
        name.bytes()
            .fold(round.wrapping_mul(2654435761), |held, byte| {
                held.wrapping_mul(31).wrapping_add(byte as usize)
            })
    }
}

/// Every law, over every fixture of the frozen corpora.
///
/// One line is rewritten by both sides, so the merge has a collision to
/// resolve, and one more by each side alone, so it has uncontested changes to
/// carry. Everything else is left exactly as the vendor wrote it, which is what
/// makes this worth running: the folds, the parameter spellings and the
/// property orders are ones nobody here chose.
#[test]
fn the_laws_hold_over_the_corpora() {
    let mut merged_fixtures = 0usize;
    let mut collisions = 0usize;
    let mut compared = 0usize;

    for (name, input) in corpus::fixtures() {
        if IcalCst::parse(&input).is_err() {
            continue;
        }

        let round_trips = IcalCst::parse(&input)
            .map(|held| held.to_bytes() == input)
            .unwrap_or(false);

        let blocks = corpus::blocks(&input);
        let editable = corpus::editable(&blocks);

        if editable.len() < 2 {
            continue;
        }

        for round in 0..3 {
            let spread = corpus::spread(&name, round);
            let shared = editable[spread % editable.len()];
            let only_left = editable[(spread / 7 + 1) % editable.len()];
            let only_right = editable[(spread / 13 + 2) % editable.len()];

            let mut left = vec![shared, only_left];
            let mut right = vec![shared, only_right];
            left.sort_unstable();
            left.dedup();
            right.sort_unstable();
            right.dedup();

            let left = corpus::edited(&input, &blocks, &left, "-left");
            let right = corpus::edited(&input, &blocks, &right, "-right");

            let Some(merged) = run(&input, &left, &right) else {
                panic!("{name} stopped parsing once edited");
            };

            merged_fixtures += 1;

            if !merged.report.conflicts.is_empty() {
                collisions += 1;
            }

            let bytes = bytes(&merged.report);
            let reparsed = IcalCst::parse(&bytes)
                .unwrap_or_else(|error| panic!("{name} merged into something unreadable: {error}"));

            assert_eq!(
                reparsed.to_bytes(),
                bytes,
                "{name} merged into something that does not survive a reparse"
            );

            if let Err(why) = completeness(&merged) {
                panic!("{name}, round {round}: {why}");
            }

            if let Err(why) = completeness_of_records(&merged) {
                panic!("{name}, round {round}: {why}");
            }

            // NOTE: A fixture the parser normalises rather than reproduces
            // byte for byte has nothing to say about byte preservation:
            // its own lines do not come back unchanged from a parse.
            for (at, block) in blocks.iter().enumerate() {
                if !round_trips || at == shared || at == only_left || at == only_right {
                    continue;
                }

                let held = &input[block.start..block.end];

                if !windows(&bytes, held) {
                    panic!(
                        "{name}, round {round}: an untouched line lost its bytes: {}",
                        String::from_utf8_lossy(held)
                    );
                }
            }

            if !comparable(&merged) {
                continue;
            }

            compared += 1;

            let reference = reference::merge(&merged.base, &merged.left, &merged.right);

            assert_eq!(
                merged.merged, reference.merged,
                "{name}, round {round}: the merged content differs from the reference"
            );

            if let Err(why) = agrees_on_conflicts(&merged, &reference.contested) {
                panic!("{name}, round {round}: {why}");
            }
        }
    }

    println!(
        "corpus merges {merged_fixtures}, with a conflict {collisions}, \
         comparable to the reference {compared}"
    );

    assert!(
        merged_fixtures > 200,
        "the corpora stopped feeding the merge: {merged_fixtures}"
    );
    assert!(
        collisions > merged_fixtures / 4,
        "the corpus edits stopped colliding: {collisions}/{merged_fixtures}"
    );
}

/// Whether a byte slice holds another.
fn windows(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty()
        || (haystack.len() >= needle.len()
            && haystack.windows(needle.len()).any(|held| held == needle))
}

/// A base holding two attendees, the shape a positional index cannot survive.
const TWO_ATTENDEES: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     BEGIN:VEVENT\r\n\
     UID:event-1@example.com\r\n\
     DTSTAMP:20260101T000000Z\r\n\
     SUMMARY:Weekly sync\r\n\
     ATTENDEE;PARTSTAT=NEEDS-ACTION;CN=Ada:mailto:ada@example.com\r\n\
     ATTENDEE;PARTSTAT=NEEDS-ACTION;CN=Bob:mailto:bob@example.com\r\n\
     END:VEVENT\r\n\
     END:VCALENDAR\r\n";

/// Merge three calendars given as text, and hand back the merged text with the
/// report.
fn merged_text<'a>(base: &'a str, left: &'a str, right: &'a str) -> (String, IcalMergeReport<'a>) {
    let report = merge(base.as_bytes(), left.as_bytes(), right.as_bytes())
        .expect("three readable calendars");
    let merged = String::from_utf8(bytes(&report)).expect("valid UTF-8");

    (merged, report)
}

/// Where both sides add a property the base lacked, the left side's wins and
/// replaces the one it beat, so the merged event holds one `LOCATION`.
///
/// Appending both would give a `VEVENT` two `LOCATION` lines, which RFC 5545
/// 3.6.1 forbids and this crate's own `validate` refuses, and would make the
/// merge non-idempotent.
#[test]
fn both_sides_adding_one_name_keeps_the_left_one() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                SUMMARY:Sync\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let left = base.replace("SUMMARY:Sync\r\n", "SUMMARY:Sync\r\nLOCATION:Room A\r\n");
    let right = base.replace("SUMMARY:Sync\r\n", "SUMMARY:Sync\r\nLOCATION:Room B\r\n");

    let (merged, report) = merged_text(base, &left, &right);

    assert!(merged.contains("LOCATION:Room A"));
    assert!(!merged.contains("LOCATION:Room B"));
    assert_eq!(report.conflicts.len(), 1);
    assert_eq!(merged.matches("LOCATION:").count(), 1);
}

/// The same for a whole component, which a `VALARM` addressed by its position
/// makes the harder case.
#[test]
fn both_sides_adding_one_alarm_keeps_the_left_one() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                SUMMARY:Sync\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let alarm = |trigger: &str, action: &str| {
        base.replace(
            "END:VEVENT",
            &format!(
                "BEGIN:VALARM\r\nTRIGGER:{trigger}\r\nACTION:{action}\r\nEND:VALARM\r\nEND:VEVENT"
            ),
        )
    };
    let left = alarm("-PT15M", "DISPLAY");
    let right = alarm("-PT30M", "AUDIO");

    let (merged, report) = merged_text(base, &left, &right);

    assert_eq!(report.conflicts.len(), 1);
    assert_eq!(merged.matches("BEGIN:VALARM").count(), 1);
    assert!(merged.contains("TRIGGER:-PT15M"));
}

/// A reply belongs to the person who wrote it, whatever the other side did to
/// the list they both sit in.
///
/// An `ATTENDEE` is its calendar user address, so a side that replaced Ada
/// with Bob removed a person rather than renaming one, and Ada.s answer meets
/// that removal rather than Bob.s line.
#[test]
fn a_reply_never_lands_on_another_attendee() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                ATTENDEE;PARTSTAT=NEEDS-ACTION;CN=Ada:mailto:ada@example.com\r\n\
                END:VEVENT\r\nEND:VCALENDAR\r\n";
    let left = base.replace(
        "ATTENDEE;PARTSTAT=NEEDS-ACTION;CN=Ada:mailto:ada@example.com",
        "ATTENDEE;PARTSTAT=NEEDS-ACTION;CN=Bob:mailto:bob@example.com",
    );
    let right = base.replace("PARTSTAT=NEEDS-ACTION;CN=Ada", "PARTSTAT=TENTATIVE;CN=Ada");

    let (merged, _) = merged_text(base, &left, &right);

    assert!(
        !merged.contains("PARTSTAT=TENTATIVE;CN=Bob"),
        "Ada's reply was written onto Bob:\n{merged}"
    );
}

/// Matching normalises and writing is exact, so a calendar address written in
/// another case is the same person and is still written back as it arrived.
#[test]
fn an_identity_meets_the_other_case_it_was_written_in() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                ATTENDEE;PARTSTAT=NEEDS-ACTION:MAILTO:Ada@Example.com\r\n\
                END:VEVENT\r\nEND:VCALENDAR\r\n";
    let left = base.replace("PARTSTAT=NEEDS-ACTION", "PARTSTAT=NEEDS-ACTION;CN=Ada");
    let right = base
        .replace("MAILTO:Ada@Example.com", "mailto:ada@example.com")
        .replace("PARTSTAT=NEEDS-ACTION", "PARTSTAT=ACCEPTED");

    let (merged, report) = merged_text(base, &left, &right);

    assert_eq!(
        merged.matches("ATTENDEE").count(),
        1,
        "one address became two attendees:\n{merged}"
    );
    assert!(
        merged.contains("PARTSTAT=ACCEPTED"),
        "Ada's answer did not land:\n{merged}"
    );
    assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
}

/// Removing one attendee does not make another attendee's contested reply
/// disappear from the report.
///
/// Both sides answered for Bob, and Ada leaving the list beside him does not
/// stop the two answers meeting.
#[test]
fn a_removal_does_not_swallow_a_neighbours_collision() {
    let left = TWO_ATTENDEES
        .replace(
            "ATTENDEE;PARTSTAT=NEEDS-ACTION;CN=Ada:mailto:ada@example.com\r\n",
            "",
        )
        .replace("PARTSTAT=NEEDS-ACTION;CN=Bob", "PARTSTAT=ACCEPTED;CN=Bob");
    let right = TWO_ATTENDEES.replace("PARTSTAT=NEEDS-ACTION;CN=Bob", "PARTSTAT=DECLINED;CN=Bob");

    let (_, report) = merged_text(TWO_ATTENDEES, &left, &right);

    assert_eq!(
        report.conflicts.len(),
        1,
        "Bob's two replies collided and nothing said so"
    );
}

/// A replayed change never turns into a second copy of a property that is
/// already there.
///
/// The replay resolves Bob by his address, so it finds the line the left side
/// kept rather than falling through to the branch that restores a removed one.
#[test]
fn a_removal_does_not_duplicate_a_neighbour() {
    let left = TWO_ATTENDEES
        .replace(
            "ATTENDEE;PARTSTAT=NEEDS-ACTION;CN=Ada:mailto:ada@example.com\r\n",
            "",
        )
        .replace("PARTSTAT=NEEDS-ACTION;CN=Bob", "PARTSTAT=ACCEPTED;CN=Bob");
    let right = TWO_ATTENDEES.replace("PARTSTAT=NEEDS-ACTION;CN=Bob", "PARTSTAT=DECLINED;CN=Bob");

    let (merged, _) = merged_text(TWO_ATTENDEES, &left, &right);

    assert_eq!(
        merged.matches("mailto:bob@example.com").count(),
        1,
        "Bob is in the merged calendar twice:\n{merged}"
    );
}

/// Removing a component does not quietly take the other side's work in its
/// descendants with it.
///
/// A component removal meets every action nested under it, at any depth, and
/// not only the ones addressed to the component itself.
#[test]
fn a_removed_component_does_not_swallow_a_nested_edit() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                SUMMARY:Sync\r\nBEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT10M\r\n\
                END:VALARM\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let left = base.replace("TRIGGER:-PT10M", "TRIGGER:-PT20M");
    let right = base.replace(
        "BEGIN:VEVENT\r\nUID:e1\r\nSUMMARY:Sync\r\nBEGIN:VALARM\r\n\
         ACTION:DISPLAY\r\nTRIGGER:-PT10M\r\nEND:VALARM\r\nEND:VEVENT\r\n",
        "",
    );

    let (_, report) = merged_text(base, &left, &right);

    assert_eq!(
        report.conflicts.len(),
        1,
        "an edit inside the removed event vanished without a word"
    );
}

/// A whole-property removal and a parameter change on that property are a
/// collision, and are reported as one.
///
/// The removal takes the answer away with the line it removes, so the two meet
/// and the answered line is what survives.
#[test]
fn a_removal_and_a_parameter_edit_are_reported() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:ada@example.com\r\n\
                END:VEVENT\r\nEND:VCALENDAR\r\n";
    let answered = base.replace("PARTSTAT=NEEDS-ACTION", "PARTSTAT=ACCEPTED");
    let dropped = base.replace(
        "ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:ada@example.com\r\n",
        "",
    );

    let (merged, report) = merged_text(base, &answered, &dropped);

    assert!(
        merged.contains("PARTSTAT=ACCEPTED"),
        "Ada's answer was dropped by a removal of her line"
    );
    assert_eq!(report.conflicts.len(), 1, "the collision went unreported");
}

/// Merging a side with itself reports nothing: two people who wrote the same
/// thing are not two people disagreeing.
///
/// A collision is compared by the values two actions carry and not only by the
/// field they occupy, so an identical edit on both sides is no divergence.
#[test]
fn merging_a_side_with_itself_reports_nothing() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                SUMMARY:Sync\r\nLOCATION:Room A\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let edited = base
        .replace("SUMMARY:Sync", "SUMMARY:Sync moved")
        .replace("LOCATION:Room A\r\n", "");

    let (_, report) = merged_text(base, &edited, &edited);

    assert!(
        report.conflicts.is_empty(),
        "two sides that agreed were reported as colliding: {:?}",
        report.conflicts
    );
}

/// A property carrying one parameter name twice does not make a merge report a
/// change nobody made.
///
/// Parameters are matched by name plus their position among the same-named
/// ones, so the second `RSVP` of a line compares against the second. Found by
/// the merge fuzz target on the first pass over its seed corpus.
#[test]
fn a_repeated_parameter_is_not_a_change() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\
                SUMMARY;RSVP=TRUE;RSVP=FALSE:Planning\r\nEND:VCALENDAR\r\n";

    let (merged, report) = merged_text(base, base, base);

    assert_eq!(merged, base, "merging a calendar with itself changed it");
    assert!(
        report.left.is_empty(),
        "a side that changed nothing was reported to have: {:?}",
        report.left
    );
    assert!(
        report.right.is_empty(),
        "a side that changed nothing was reported to have: {:?}",
        report.right
    );
    assert!(
        report.conflicts.is_empty(),
        "two sides that did nothing collided"
    );
}

/// Whatever the sides hold, the merged calendar can be read back.
///
/// A bare, envelope-less record holds `BEGIN` and `END` lines where a calendar
/// holds an envelope, and the merge treats them as the envelope rather than
/// copying a structural keyword into the middle of a well-formed calendar. The
/// last line of a truncated one carries no line ending, and copied without one
/// it would swallow the line it lands in front of. Both found by the merge fuzz
/// target.
#[test]
fn a_merge_never_emits_a_calendar_that_cannot_be_read() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";

    for bare in [
        "VERSION:2.0\r\nBEGIN:VEVENT\r\nUID:x\r\n",
        "VERSION:2.0\r\nEND:VCALENDAR\r\n",
        "VERSION:2.0\r\nSUMMARY:truncated",
    ] {
        let report = merge(base.as_bytes(), base.as_bytes(), bare.as_bytes())
            .expect("three readable calendars");

        let merged = bytes(&report);
        let reparsed = IcalCst::parse(&merged);

        assert!(
            reparsed.is_ok(),
            "the merged calendar does not parse: {}",
            String::from_utf8_lossy(&merged)
        );
        assert_eq!(
            reparsed.unwrap().to_bytes(),
            merged,
            "the merged calendar loses bytes on a reparse"
        );
    }
}

/// A merge against a side that changed nothing yields the other side, exactly.
///
/// This is the sharpest form of the identity rule: only one side edited
/// anything, so there is nothing to reconcile, and a merge addressing an
/// attendee by a position would invent a person out of two.
#[test]
fn an_unchanged_side_yields_the_other_exactly() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                ATTENDEE;CN=Ada:mailto:ada@example.com\r\n\
                ATTENDEE;CN=Zoe:mailto:zoe@example.com\r\n\
                END:VEVENT\r\nEND:VCALENDAR\r\n";
    let right = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                 ATTENDEE;CN=Zoe:mailto:zoe@example.com\r\n\
                 ATTENDEE;CN=Bob:mailto:bob@example.com\r\n\
                 END:VEVENT\r\nEND:VCALENDAR\r\n";

    let (merged, _) = merged_text(base, base, right);

    assert_eq!(merged, right, "the merged calendar is not the right side");
    assert!(
        !merged.contains("CN=Bob:mailto:zoe@example.com"),
        "the merge invented an attendee: {merged}"
    );
}

/// A calendar holding one `UID` twice merges with itself in silence.
///
/// Two components at one path are not one component seen twice: matching both
/// of them against the same one on the other side would report the difference
/// between them as a change a side made. Found by the merge fuzz target.
#[test]
fn a_uid_written_twice_is_two_components() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\
                BEGIN:VEVENT\r\nUID:e1\r\nSUMMARY:First\r\nEND:VEVENT\r\n\
                BEGIN:VEVENT\r\nUID:e1\r\nSUMMARY:Second\r\nEND:VEVENT\r\n\
                END:VCALENDAR\r\n";
    let edited = base.replace("SUMMARY:First", "SUMMARY:Moved");

    let (merged, report) = merged_text(base, &edited, &edited);

    assert_eq!(merged, edited, "merging a side with itself changed it");
    assert!(
        report.conflicts.is_empty(),
        "two sides that agreed collided: {:?}",
        report.conflicts
    );
}

/// A calendar address written on two attendees tells neither of them apart, so
/// the two fall back to their positions rather than becoming one attendee.
///
/// An identity that does not distinguish is no identity. Found by the merge
/// fuzz target, on a line whose value held a comma.
#[test]
fn a_repeated_calendar_address_is_not_an_identity() {
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                ATTENDEE;CN=Ada:mailto:ada@example.com\r\n\
                ATTENDEE;CN=Ada at home:mailto:ada@example.com\r\n\
                END:VEVENT\r\nEND:VCALENDAR\r\n";
    let edited = base.replace("CN=Ada at home", "CN=Ada elsewhere");

    let (merged, report) = merged_text(base, &edited, &edited);

    assert_eq!(merged, edited, "merging a side with itself changed it");
    assert!(
        report.conflicts.is_empty(),
        "two sides that agreed collided: {:?}",
        report.conflicts
    );
}
