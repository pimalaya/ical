//! # Recurrence set
//!
//! The occurrences a *component* denotes, not the ones a single rule does.
//!
//! RFC 5545 3.8.5 builds the set a `VEVENT` or `VTODO` actually happens on out
//! of five properties, plus the overrides that sit in sibling components:
//! `DTSTART` and every `RRULE` and `RDATE` add instances, every `EXDATE` and
//! `EXRULE` take them away, and a component carrying a `RECURRENCE-ID` replaces
//! one. [`IcalRecurSet`] holds those pieces and [`IcalRecurSetExpand`] walks
//! them.
//!
//! ## Identity, and the order it comes in
//!
//! Every occurrence has two times. Its **identity** is the time the rules place
//! it at, which is what a `RECURRENCE-ID` names and what an `EXDATE` removes.
//! Its **start** is when it actually happens, which is the identity unless an
//! override moved it.
//!
//! Occurrences come out in the chronological order of their *identity*, which is
//! what keeps the walk lazy: nothing is buffered, so an endless rule can be taken
//! from without running it to its end. An override that moves an instance is
//! emitted in the place of the instance it replaces, so its start can fall out of
//! order. A caller that needs starts in order sorts a window of them, which is a
//! decision about a window, not about the walk.
//!
//! ## Civil, like everything else here
//!
//! Nothing here resolves a time zone. `DTSTART`, `RDATE`, `EXDATE` and
//! `RECURRENCE-ID` are read as the civil times they spell, and a `TZID`
//! parameter is ignored, exactly as [expansion](crate::recur::expand) ignores it.

use alloc::{vec, vec::Vec};

use crate::{
    component::IcalComponent,
    param::IcalParam,
    prop::{IcalPropKind, IcalPropName},
    recur::{IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand},
    value::IcalValue,
};

/// One occurrence of a recurrence set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IcalRecurOccurrence {
    /// The instance identity: the time the rules place this occurrence at, and
    /// the value a `RECURRENCE-ID` would carry to name it.
    pub id: IcalRecurDateTime,
    /// When the occurrence actually starts. The same as
    /// [`id`](Self::id) unless an override moved it.
    pub start: IcalRecurDateTime,
    /// The index into [`IcalRecurSet::overrides`] of the override that replaced
    /// this instance, if one did.
    pub over: Option<usize>,
}

/// A component that replaces one instance of a set, keyed by the identity its
/// `RECURRENCE-ID` names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IcalRecurOverride {
    /// The identity this override replaces, from its `RECURRENCE-ID`.
    pub id: IcalRecurDateTime,
    /// The overriding start, from the override component's own `DTSTART`.
    pub start: IcalRecurDateTime,
    /// Whether the override carried `RANGE=THISANDFUTURE`, which shifts this
    /// instance and every later one by the same offset.
    pub this_and_future: bool,
}

/// The recurrence set of one component: what adds to it, what takes away from
/// it, and what replaces an instance of it.
///
/// Build one from a decoded component with
/// [`of_component`](Self::of_component), or by hand, and walk it with
/// [`expand`](Self::expand).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalRecurSet {
    /// The `DTSTART`, always the first instance of the set (RFC 5545 3.8.2.4).
    pub start: Option<IcalRecurDateTime>,
    /// Every `RRULE`.
    pub rules: Vec<IcalRecurRule>,
    /// Every date named by an `RDATE`, in order. A period item contributes its
    /// start.
    pub dates: Vec<IcalRecurDateTime>,
    /// Every `EXRULE`, deprecated by RFC 5545 but still on the wire.
    pub exrules: Vec<IcalRecurRule>,
    /// Every date named by an `EXDATE`, in order.
    pub exdates: Vec<IcalRecurDateTime>,
    /// The overrides that replace instances, in order of identity.
    pub overrides: Vec<IcalRecurOverride>,
}

impl IcalRecurSet {
    /// Read the set a decoded component denotes, its overrides aside.
    ///
    /// A component with no `DTSTART` and no `RDATE` denotes nothing, and comes
    /// back empty rather than as an error: this is the liberal side of the
    /// crate, and a rule that names no start is simply a rule nobody can
    /// expand. A malformed date or rule is skipped for the same reason.
    pub fn of_component(component: &IcalComponent<'_>) -> Self {
        let mut set = Self::default();

        for prop in &component.props {
            let IcalPropName::Kind(kind) = prop.name else {
                continue;
            };

            match kind {
                IcalPropKind::DtStart => set.start = date_of(&prop.value),
                IcalPropKind::RRule => set.rules.extend(rule_of(&prop.value)),
                IcalPropKind::ExRule => set.exrules.extend(rule_of(&prop.value)),
                IcalPropKind::RDate => set.dates.extend(dates_of(&prop.value)),
                IcalPropKind::ExDate => set.exdates.extend(dates_of(&prop.value)),
                _ => {}
            }
        }

        set.dates.sort_unstable();
        set.dates.dedup();
        set.exdates.sort_unstable();
        set.exdates.dedup();

        set
    }

    /// Add the override a sibling component carrying a `RECURRENCE-ID` states.
    ///
    /// A component with no `RECURRENCE-ID`, or with no `DTSTART` to move the
    /// instance to, is not an override and is ignored.
    pub fn with_override(&mut self, component: &IcalComponent<'_>) -> &mut Self {
        let mut id = None;
        let mut start = None;
        let mut this_and_future = false;

        for prop in &component.props {
            let IcalPropName::Kind(kind) = prop.name else {
                continue;
            };

            match kind {
                IcalPropKind::RecurrenceId => {
                    id = date_of(&prop.value);
                    this_and_future = prop.params.iter().any(|param| {
                        matches!(param, IcalParam::Range(range) if range.eq_ignore_ascii_case("THISANDFUTURE"))
                    });
                }
                IcalPropKind::DtStart => start = date_of(&prop.value),
                _ => {}
            }
        }

        if let (Some(id), Some(start)) = (id, start) {
            self.overrides.push(IcalRecurOverride {
                id,
                start,
                this_and_future,
            });
            self.overrides.sort_unstable_by_key(|over| over.id);
        }

        self
    }

    /// The set a whole calendar denotes for one `UID`: the series component,
    /// plus every sibling that overrides an instance of it.
    ///
    /// The series is the component carrying that `UID` with no
    /// `RECURRENCE-ID`; every other one carrying it is an override.
    pub fn of_uid(components: &[IcalComponent<'_>], uid: &str) -> Self {
        let mut set = Self::default();

        for component in components {
            if uid_of(component) != Some(uid) {
                continue;
            }

            if has(component, IcalPropKind::RecurrenceId) {
                set.with_override(component);
            } else {
                let series = Self::of_component(component);
                set.start = series.start;
                set.rules = series.rules;
                set.dates = series.dates;
                set.exrules = series.exrules;
                set.exdates = series.exdates;
            }
        }

        set
    }

    /// Walk the set, lazily, in identity order.
    pub fn expand(&self) -> IcalRecurSetExpand<'_> {
        let start = self.start;

        IcalRecurSetExpand {
            set: self,
            streams: self
                .rules
                .iter()
                .filter_map(|rule| start.map(|start| IcalRecurExpand::new(rule.clone(), start)))
                .collect(),
            heads: vec![None; self.rules.len()],
            primed: false,
            // NOTE: The literal sources: DTSTART, the RDATEs, and the identity
            // of every override, which is an instance whether or not a rule
            // generates it.
            literals: {
                let mut literals: Vec<IcalRecurDateTime> = start.into_iter().collect();
                literals.extend(self.dates.iter().copied());
                literals.extend(self.overrides.iter().map(|over| over.id));
                literals.sort_unstable();
                literals.dedup();
                literals
            },
            literal: 0,
            exrules: self
                .exrules
                .iter()
                .filter_map(|rule| start.map(|start| IcalRecurExpand::new(rule.clone(), start)))
                .collect(),
            exheads: vec![None; self.exrules.len()],
            last: None,
        }
    }
}

/// The lazy walk of an [`IcalRecurSet`], in identity order.
///
/// A k-way merge over the rule expansions and the literal dates, with the
/// exclusions applied as it goes: an `EXDATE` is a membership test on a sorted
/// list, an `EXRULE` is another lazy stream advanced in step. Nothing is
/// materialised beyond the literal lists, which are literal on the wire too.
pub struct IcalRecurSetExpand<'a> {
    set: &'a IcalRecurSet,
    streams: Vec<IcalRecurExpand>,
    heads: Vec<Option<IcalRecurDateTime>>,
    primed: bool,
    literals: Vec<IcalRecurDateTime>,
    literal: usize,
    exrules: Vec<IcalRecurExpand>,
    exheads: Vec<Option<IcalRecurDateTime>>,
    last: Option<IcalRecurDateTime>,
}

impl Iterator for IcalRecurSetExpand<'_> {
    type Item = IcalRecurOccurrence;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.next_id()?;

            if self.excluded(id) {
                continue;
            }

            let over = self.set.overrides.iter().position(|over| over.id == id);

            let start = match over {
                Some(index) => self.set.overrides[index].start,
                None => IcalRecurDateTime::from_seconds(id.seconds() + self.shift(id)),
            };

            return Some(IcalRecurOccurrence { id, start, over });
        }
    }
}

impl IcalRecurSetExpand<'_> {
    /// The next identity in chronological order, deduplicated across sources.
    fn next_id(&mut self) -> Option<IcalRecurDateTime> {
        loop {
            if !self.primed {
                for (index, stream) in self.streams.iter_mut().enumerate() {
                    self.heads[index] = stream.next();
                }
                self.primed = true;
            }

            let from_rules = self.heads.iter().flatten().min().copied();
            let from_literals = self.literals.get(self.literal).copied();

            let next = match (from_rules, from_literals) {
                (Some(rule), Some(literal)) => rule.min(literal),
                (Some(rule), None) => rule,
                (None, Some(literal)) => literal,
                (None, None) => return None,
            };

            // NOTE: Consume every source sitting on it, so an instance a rule
            // and an RDATE both name is yielded once.
            for (index, head) in self.heads.iter_mut().enumerate() {
                if *head == Some(next) {
                    *head = self.streams[index].next();
                }
            }
            if from_literals == Some(next) {
                self.literal += 1;
            }

            // NOTE: A rule and a literal can still collide across iterations
            // when a stream repeats a value; the last-yielded guard makes the
            // walk strictly increasing whatever the sources do.
            if self.last == Some(next) {
                continue;
            }

            self.last = Some(next);
            return Some(next);
        }
    }

    /// Whether an identity is excluded, by an `EXDATE` or an `EXRULE`.
    fn excluded(&mut self, id: IcalRecurDateTime) -> bool {
        if self.set.exdates.binary_search(&id).is_ok() {
            return true;
        }

        for (index, stream) in self.exrules.iter_mut().enumerate() {
            // NOTE: Advance this exception stream up to the candidate: it is
            // sorted, so anything it has already passed can never match again.
            while self.exheads[index].is_none_or(|head| head < id) {
                match stream.next() {
                    Some(next) => self.exheads[index] = Some(next),
                    None => break,
                }
            }

            if self.exheads[index] == Some(id) {
                return true;
            }
        }

        false
    }

    /// The offset every `RANGE=THISANDFUTURE` override in force at `id`
    /// applies, in seconds. The latest one wins, as a later override restates
    /// the shift rather than compounding it.
    fn shift(&self, id: IcalRecurDateTime) -> i64 {
        self.set
            .overrides
            .iter()
            .rfind(|over| over.this_and_future && over.id <= id)
            .map(|over| over.start.seconds() - over.id.seconds())
            .unwrap_or(0)
    }
}

/// The civil date a date-ish value names, when it names one.
fn date_of(value: &IcalValue<'_>) -> Option<IcalRecurDateTime> {
    let text = match value {
        IcalValue::Date(date) => &date.0,
        IcalValue::DateTime(date) => &date.0,
        IcalValue::DateTimeList(dates) => dates.0.first()?,
        _ => return None,
    };

    IcalRecurDateTime::parse(text).ok()
}

/// Every civil date a list value names. A period item (`start/end` or
/// `start/duration`, which `RDATE` admits) contributes its start.
fn dates_of(value: &IcalValue<'_>) -> Vec<IcalRecurDateTime> {
    let items: &[_] = match value {
        IcalValue::DateTimeList(dates) => &dates.0,
        // NOTE: A single-valued RDATE or EXDATE, however it was built.
        other => return date_of(other).into_iter().collect(),
    };

    items
        .iter()
        .filter_map(|item| {
            let start = item.split('/').next().unwrap_or(item);
            IcalRecurDateTime::parse(start).ok()
        })
        .collect()
}

/// The rule a recurrence value states, when it states a readable one.
fn rule_of(value: &IcalValue<'_>) -> Option<IcalRecurRule> {
    let IcalValue::Recur(recur) = value else {
        return None;
    };

    IcalRecurRule::parse(&recur.0).ok()
}

/// The `UID` of a component, if it carries one.
fn uid_of<'a>(component: &'a IcalComponent<'_>) -> Option<&'a str> {
    component.props.iter().find_map(|prop| {
        if !matches!(prop.name, IcalPropName::Kind(IcalPropKind::Uid)) {
            return None;
        }

        match &prop.value {
            IcalValue::Text(text) => Some(&*text.0),
            _ => None,
        }
    })
}

/// Whether a component carries a property of the given kind.
fn has(component: &IcalComponent<'_>, kind: IcalPropKind) -> bool {
    component
        .props
        .iter()
        .any(|prop| matches!(prop.name, IcalPropName::Kind(k) if k == kind))
}
