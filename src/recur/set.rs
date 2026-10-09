//! # Recurrence set
//!
//! The occurrences a *component* denotes, not the ones a single rule does.
//!
//! RFC 5545 3.8.5 builds the set a `VEVENT` or `VTODO` actually happens on
//! out of five properties, plus the overrides that sit in sibling components.
//!
//! `DTSTART` and every `RRULE` and `RDATE` add instances, every `EXDATE` and
//! `EXRULE` take them away, and a component carrying a `RECURRENCE-ID`
//! replaces one. [`IcalRecurSet`] holds those pieces and
//! [`IcalRecurSetExpand`] walks them.
//!
//! ## Identity, and the order it comes in
//!
//! Every occurrence has two times. Its **identity** is the time the rules
//! place it at, which is what a `RECURRENCE-ID` names and what an `EXDATE`
//! removes. Its **start** is when it actually happens, which is the identity
//! unless an override moved it.
//!
//! Occurrences come out in the chronological order of their *identity*, which
//! is what keeps the walk lazy: nothing is buffered, so an endless rule can
//! be taken from without running it to its end.
//!
//! An override that moves an instance is emitted in the place of the instance
//! it replaces, so its start can fall out of order. A caller that needs
//! starts in order sorts a window of them, which is a decision about a
//! window, not about the walk.
//!
//! ## Civil, on one clock
//!
//! Expansion is civil, as [expansion](crate::recur::expand) is: every time of
//! a set is a civil time on one clock, the one its `DTSTART` is told in
//! ([`IcalRecurSet::zone`]).
//!
//! A time written on another clock is brought onto that one as the set is
//! read, when the zones that define both are given: a UTC `UNTIL`, which RFC
//! 5545 3.3.10 requires beside a zoned `DTSTART`, and an `RDATE`, `EXDATE` or
//! `RECURRENCE-ID` in UTC or under another `TZID` (3.8.5.1, 3.8.4.4), each
//! read as the written `DATE-TIME` it is (3.3.5).
//! [`of_uid`](IcalRecurSet::of_uid) finds the zones among the calendar's own
//! components; [`of_component_in`](IcalRecurSet::of_component_in) and
//! [`with_override_in`](IcalRecurSet::with_override_in) take them. Without
//! them, or on a floating clock, a time is compared as it is written.

use alloc::{string::String, vec, vec::Vec};

use crate::{
    component::IcalComponent,
    param::IcalParam,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    recur::{IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand},
    tz::IcalTz,
    value::IcalValue,
};

/// The clock a civil date-time is told on (RFC 5545 3.3.4, 3.3.5).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum IcalRecurZone {
    /// A date, or a local time with no zone, read on whichever clock reads it.
    #[default]
    Floating,
    /// A UTC time, written with a trailing `Z`.
    Utc,
    /// A local time in the zone a `TZID` parameter names, verbatim.
    Tz(String),
}

impl IcalRecurZone {
    /// The clock a date property's value is told on: UTC for a value ending in
    /// `Z`, else the zone its `TZID` names, else floating.
    fn of(params: &[IcalParam<'_>], value: &str) -> Self {
        if value.ends_with(['Z', 'z']) {
            return Self::Utc;
        }

        params
            .iter()
            .find_map(|param| match param {
                IcalParam::TzId(tzid) => Some(Self::Tz(String::from(tzid.as_ref()))),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// The same civil time told on another clock, through the zones that
    /// define both, each read as a written `DATE-TIME` (RFC 5545 3.3.5).
    ///
    /// Unchanged when either clock is floating or a zone is not among `zones`:
    /// comparing the times as written is then all there is to do.
    fn tell(&self, local: IcalRecurDateTime, to: &Self, zones: &[IcalTz]) -> IcalRecurDateTime {
        let find = |id: &str| zones.iter().find(|zone| zone.id == id);

        let instant = match self {
            _ if self == to => return local,
            Self::Floating => return local,
            Self::Utc => local.seconds(),
            Self::Tz(id) => match find(id) {
                Some(zone) => zone.resolve(local).literal_instant(local),
                None => return local,
            },
        };

        match to {
            Self::Floating => local,
            Self::Utc => IcalRecurDateTime::from_seconds(instant),
            Self::Tz(id) => find(id).map_or(local, |zone| zone.local(instant)),
        }
    }
}

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

impl IcalRecurOverride {
    /// Read the override a component carrying a `RECURRENCE-ID` states, its
    /// identity and its start told on the series' clock, `zone`, through the
    /// zones `zones` defines (see the [module](self) header).
    ///
    /// `None` for a component with no `RECURRENCE-ID` or no `DTSTART` to move
    /// the instance to, which is not an override.
    pub fn of_component(
        component: &IcalComponent<'_>,
        zone: &IcalRecurZone,
        zones: &[IcalTz],
    ) -> Option<Self> {
        let mut id = None;
        let mut start = None;
        let mut this_and_future = false;

        for prop in &component.props {
            let IcalPropName::Kind(kind) = prop.name else {
                continue;
            };

            match kind {
                IcalPropKind::RecurrenceId => {
                    id = date_of(prop, zone, zones);
                    this_and_future = prop.params.iter().any(|param| {
                        matches!(param, IcalParam::Range(range) if range.eq_ignore_ascii_case("THISANDFUTURE"))
                    });
                }
                IcalPropKind::DtStart => start = date_of(prop, zone, zones),
                _ => {}
            }
        }

        Some(Self {
            id: id?,
            start: start?,
            this_and_future,
        })
    }
}

/// The recurrence set of one component: what adds, subtracts and overrides.
///
/// Build one from a decoded component with
/// [`of_component`](Self::of_component), or by hand, and walk it with
/// [`expand`](Self::expand).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalRecurSet {
    /// The clock every time of the set is told on: its `DTSTART`'s.
    pub zone: IcalRecurZone,
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
    /// Read the set a decoded component denotes, its overrides aside, every
    /// time compared as it is written.
    ///
    /// A component with no `DTSTART` and no `RDATE` denotes nothing and comes
    /// back empty rather than as an error: this is the liberal side of the
    /// crate. A malformed date or rule is skipped for the same reason.
    pub fn of_component(component: &IcalComponent<'_>) -> Self {
        Self::of_component_in(component, &[])
    }

    /// Read the set a decoded component denotes, its overrides aside, a time
    /// written on another clock than its `DTSTART`'s told on that one through
    /// the zones `zones` defines (see the [module](self) header).
    pub fn of_component_in(component: &IcalComponent<'_>, zones: &[IcalTz]) -> Self {
        let mut set = Self::default();

        let start = component
            .props
            .iter()
            .find(|prop| matches!(prop.name, IcalPropName::Kind(IcalPropKind::DtStart)));

        if let Some(start) = start
            && let Some(text) = text_of(&start.value)
        {
            set.zone = IcalRecurZone::of(&start.params, text);
            set.start = IcalRecurDateTime::parse(text).ok();
        }

        for prop in &component.props {
            let IcalPropName::Kind(kind) = prop.name else {
                continue;
            };

            match kind {
                IcalPropKind::RRule => set.rules.extend(rule_of(prop, &set.zone, zones)),
                IcalPropKind::ExRule => set.exrules.extend(rule_of(prop, &set.zone, zones)),
                IcalPropKind::RDate => set.dates.extend(dates_of(prop, &set.zone, zones)),
                IcalPropKind::ExDate => set.exdates.extend(dates_of(prop, &set.zone, zones)),
                _ => {}
            }
        }

        set.dates.sort_unstable();
        set.dates.dedup();
        set.exdates.sort_unstable();
        set.exdates.dedup();

        set
    }

    /// Add the override a sibling component carrying a `RECURRENCE-ID` states,
    /// its times compared as they are written.
    ///
    /// A component with no `RECURRENCE-ID`, or with no `DTSTART` to move the
    /// instance to, is not an override and is ignored.
    pub fn with_override(&mut self, component: &IcalComponent<'_>) -> &mut Self {
        self.with_override_in(component, &[])
    }

    /// Add the override a sibling component states, its `RECURRENCE-ID` and
    /// its `DTSTART` told on the set's clock through the zones `zones` defines,
    /// as [`IcalRecurOverride::of_component`] reads them.
    pub fn with_override_in(
        &mut self,
        component: &IcalComponent<'_>,
        zones: &[IcalTz],
    ) -> &mut Self {
        if let Some(over) = IcalRecurOverride::of_component(component, &self.zone, zones) {
            self.overrides.push(over);
            self.overrides.sort_unstable_by_key(|over| over.id);
        }

        self
    }

    /// The set a whole calendar denotes for one `UID`: the series component,
    /// plus every sibling that overrides an instance of it, every time told on
    /// the series' clock through the `VTIMEZONE`s among `components`.
    ///
    /// The series is the component carrying that `UID` with no
    /// `RECURRENCE-ID`; every other one carrying it is an override.
    pub fn of_uid(components: &[IcalComponent<'_>], uid: &str) -> Self {
        let zones: Vec<IcalTz> = components.iter().filter_map(IcalTz::of_component).collect();
        let ours = || {
            components
                .iter()
                .filter(move |component| uid_of(component) == Some(uid))
        };

        // NOTE: The series first, whatever order the calendar lists them in,
        // since an override is told on the series' clock.
        let mut set = ours()
            .rfind(|component| !has(component, IcalPropKind::RecurrenceId))
            .map(|series| Self::of_component_in(series, &zones))
            .unwrap_or_default();

        for over in ours().filter(|component| has(component, IcalPropKind::RecurrenceId)) {
            set.with_override_in(over, &zones);
        }

        set
    }

    /// Walk the set, lazily, in identity order.
    pub fn expand(&self) -> IcalRecurSetExpand<'_> {
        self.walk(None)
    }

    /// Walk the set against a time zone, dropping the instances its rules
    /// generate at local times the zone jumps over (RFC 5545 3.3.10).
    ///
    /// The filter sits on the rule streams alone. An `RDATE` does not generate
    /// an instance, it names one, so a date written into one is as deliberate
    /// as a lone `DTSTART` and is kept whatever the zone says of it.
    pub fn expand_in_zone(&self, zone: &IcalTz) -> IcalRecurSetExpand<'_> {
        self.walk(Some(zone))
    }

    /// The walk both expansions are, with and without a zone to filter by.
    fn walk(&self, zone: Option<&IcalTz>) -> IcalRecurSetExpand<'_> {
        let start = self.start;

        let stream = |rule: &IcalRecurRule| {
            let expand = IcalRecurExpand::new(rule.clone(), start?);

            Some(match zone {
                Some(zone) => expand.in_zone(zone.clone()),
                None => expand,
            })
        };

        IcalRecurSetExpand {
            set: self,
            streams: self.rules.iter().filter_map(stream).collect(),
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
            exrules: self.exrules.iter().filter_map(stream).collect(),
            exheads: vec![None; self.exrules.len()],
            last: None,
        }
    }
}

/// The lazy walk of an [`IcalRecurSet`], in identity order.
///
/// A k-way merge over the rule expansions and the literal dates, exclusions
/// applied as it goes: an `EXDATE` is a membership test, an `EXRULE` another
/// lazy stream in step. Only the wire-literal lists are ever materialised.
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

/// The text of a date-ish value: a date, a date-time, or a list's first item.
fn text_of<'v>(value: &'v IcalValue<'_>) -> Option<&'v str> {
    match value {
        IcalValue::Date(date) => Some(&date.0),
        IcalValue::DateTime(date) => Some(&date.0),
        IcalValue::DateTimeList(dates) => dates.0.first().map(|date| date.as_ref()),
        _ => None,
    }
}

/// The civil date a date-ish property names, told on `zone`'s clock.
fn date_of(
    prop: &IcalProp<'_>,
    zone: &IcalRecurZone,
    zones: &[IcalTz],
) -> Option<IcalRecurDateTime> {
    let text = text_of(&prop.value)?;
    let local = IcalRecurDateTime::parse(text).ok()?;

    Some(IcalRecurZone::of(&prop.params, text).tell(local, zone, zones))
}

/// Every civil date a list property names, told on `zone`'s clock. A period
/// item (`start/end` or `start/duration`, which `RDATE` admits) contributes
/// its start.
fn dates_of(prop: &IcalProp<'_>, zone: &IcalRecurZone, zones: &[IcalTz]) -> Vec<IcalRecurDateTime> {
    let IcalValue::DateTimeList(dates) = &prop.value else {
        // NOTE: A single-valued RDATE or EXDATE, however it was built.
        return date_of(prop, zone, zones).into_iter().collect();
    };

    dates
        .0
        .iter()
        .filter_map(|item| {
            let start = item.split('/').next().unwrap_or(item);
            let local = IcalRecurDateTime::parse(start).ok()?;

            Some(IcalRecurZone::of(&prop.params, start).tell(local, zone, zones))
        })
        .collect()
}

/// The rule a recurrence property states, when it states a readable one, its
/// `UNTIL` told on `zone`'s clock when it is written in UTC.
fn rule_of(prop: &IcalProp<'_>, zone: &IcalRecurZone, zones: &[IcalTz]) -> Option<IcalRecurRule> {
    let IcalValue::Recur(recur) = &prop.value else {
        return None;
    };

    let mut rule = IcalRecurRule::parse(&recur.0).ok()?;

    // NOTE: The parsed bound has dropped the `Z`, so the raw part says which
    // clock it was on: UTC, or the start's own (RFC 5545 3.3.10).
    let utc = recur.0.split(';').any(|part| {
        part.split_once('=').is_some_and(|(name, value)| {
            name.trim().eq_ignore_ascii_case("UNTIL") && value.trim().ends_with(['Z', 'z'])
        })
    });

    if utc && let Some(until) = rule.until {
        rule.until = Some(IcalRecurZone::Utc.tell(until, zone, zones));
    }

    Some(rule)
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
