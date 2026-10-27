//! # Occurrences as instants
//!
//! The occurrences of a series over a window of instants, each with a start
//! and an end on the timeline rather than on a wall clock.
//!
//! The [set](crate::recur::set) walk stays civil and yields identities and
//! starts told on the series' clock. This is the step after it, in the order
//! an occurrence is built: the set places it, overrides and exclusions
//! applied on one clock; only then is its start resolved through the zone its
//! clock names ([`tz`](crate::tz)), and its end placed from its own
//! component.
//!
//! [`IcalRecurTimeline`] reads one `UID` of a calendar, its series and every
//! override, through the zones it is given:
//! [`IcalTz::of_components`] for the calendar's own `VTIMEZONE`s, or, behind
//! the `tzdb` feature, `tzdb::zones` for those completed by the database.
//! [`expand`](IcalRecurTimeline::expand) walks it over a window.
//!
//! ## The end
//!
//! An occurrence ends at its component's `DTEND` (a `VTODO`'s `DUE`), read on
//! its own clock, else its start plus its `DURATION`, else at RFC 5545
//! 3.6.1's default: a day later for a `VEVENT` on a date, at its start
//! otherwise. A `DTEND` span applies to every other instance as the exact
//! number of seconds it measures (RFC 5545 3.8.5.3), an `RDATE` period's own
//! span to its instance.
//!
//! A `DURATION` is added as RFC 5545 3.3.6 adds it: its days and weeks on the
//! start's wall clock before resolving, its hours, minutes and seconds after,
//! so `P1D` across a transition lasts 23 or 25 hours and `PT24H` lasts 24.
//!
//! ## Floating times
//!
//! A floating or date time, or one under a `TZID` none of the zones defines,
//! is read as if on UTC, so its instant is its civil second count, and the
//! occurrence says so ([`IcalRecurInstant::floating`]): a consumer showing it
//! on a viewer's clock reads the civil time back with
//! [`IcalRecurDateTime::from_seconds`] and resolves it there.

use alloc::{collections::VecDeque, vec, vec::Vec};

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    prop::{IcalProp, IcalPropKind, IcalPropName},
    recur::{
        IcalRecurDateTime,
        set::{
            IcalRecurOverride, IcalRecurSet, IcalRecurSetExpand, IcalRecurZone, has, text_of,
            uid_of,
        },
    },
    tz::{IcalTz, IcalTzTransitions},
    value::{IcalValue, duration::IcalDuration},
};

/// A day, in seconds.
const DAY: i64 = 86_400;

/// How far a local clock may read from UTC, and then some: the civil bounds of
/// a walk are widened by it on both sides, before the exact filter on
/// instants.
const SLACK: i64 = 2 * DAY;

/// The first and last civil seconds a walk is bounded to, the years the rule
/// expander itself is capped at.
const EARLIEST: i64 = IcalRecurDateTime::date(0, 1, 1).seconds();
const LATEST: i64 = IcalRecurDateTime::date(10_000, 1, 1).seconds();

/// One occurrence of a series, placed on the timeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IcalRecurInstant {
    /// The instance identity, civil on the series' clock: the time the rules
    /// place it at, and the value a `RECURRENCE-ID` carries to name it.
    pub id: IcalRecurDateTime,
    /// The instant the identity names, in seconds since the Unix epoch.
    pub id_instant: i64,
    /// When the occurrence starts, in seconds since the Unix epoch.
    pub start: i64,
    /// When it ends, in seconds since the Unix epoch, never before
    /// [`start`](Self::start).
    pub end: i64,
    /// Whether an override (a component carrying a `RECURRENCE-ID` naming
    /// this instance) placed it. An instance shifted by another one's
    /// `RANGE=THISANDFUTURE` is not placed by an override of its own.
    pub over: bool,
    /// Whether the start was read on no zone, as if on UTC (see the
    /// [module](self) header).
    pub floating: bool,
    /// Whether the start is a date rather than a date-time: an all-day
    /// occurrence, from midnight to midnight on the floating clock.
    pub date: bool,
}

/// The series of one `UID` of a calendar, its overrides included, ready to be
/// walked over windows of instants.
///
/// Holds what one series needs and nothing borrowed, so it outlives the
/// calendar it was read from.
#[derive(Clone, Debug)]
pub struct IcalRecurTimeline {
    set: IcalRecurSet,
    zone: Option<IcalTz>,
    floating: bool,
    date: bool,
    span: Span,
    /// The identity of every `RDATE` period, on the series' clock, and the
    /// seconds it lasts.
    periods: Vec<(IcalRecurDateTime, i64)>,
    /// Each override placed, at the index of the set's own.
    overrides: Vec<Placed>,
    /// The one occurrence a task with a `DUE` and no `DTSTART` is.
    due: Option<Placed>,
}

impl IcalRecurTimeline {
    /// Read the series of one `UID` among a calendar's components, every
    /// time resolved through `zones`, the first zone of a name winning.
    ///
    /// The series is the component carrying that `UID` with no
    /// `RECURRENCE-ID`, and every other one carrying it overrides an instance.
    /// One with no `DTSTART` denotes nothing, its `RRULE` included, and walks
    /// empty rather than failing; an override with no series is still the
    /// instance it names.
    pub fn of_uid(components: &[IcalComponent<'_>], uid: &str, zones: &[IcalTz]) -> Self {
        let ours = || {
            components
                .iter()
                .filter(move |component| uid_of(component) == Some(uid))
        };

        // NOTE: The series first, whatever order the calendar lists them in,
        // since an override is told on the series' clock.
        let series = ours().rfind(|component| !has(component, IcalPropKind::RecurrenceId));

        let mut set = series
            .map(|series| IcalRecurSet::of_component_in(series, zones))
            .unwrap_or_default();

        let mut overrides: Vec<(IcalRecurOverride, Placed)> = ours()
            .filter(|component| has(component, IcalPropKind::RecurrenceId))
            .filter_map(|component| {
                let over = IcalRecurOverride::of_component(component, &set.zone, zones)?;
                let placed = Placed::of_component(component, over.id, zones)?;
                Some((over, placed))
            })
            .collect();

        // NOTE: Stable, so of two overrides naming one instance the first
        // listed wins on every run, the walk taking the first of a name.
        overrides.sort_by_key(|(over, _)| over.id);

        let (overs, overrides) = overrides.into_iter().unzip();
        set.overrides = overs;

        let zone = match &set.zone {
            IcalRecurZone::Tz(id) => zones.iter().find(|zone| zone.id == *id).cloned(),
            _ => None,
        };

        let floating = match &set.zone {
            IcalRecurZone::Utc => false,
            IcalRecurZone::Floating => true,
            IcalRecurZone::Tz(_) => zone.is_none(),
        };

        let date = series
            .and_then(|series| prop_of(series, IcalPropKind::DtStart))
            .is_some_and(is_date);

        let span = series.map_or(Span::Exact(0), |series| Span::of_component(series, zones));

        let periods = series.map_or_else(Vec::new, |series| periods_of(series, &set.zone, zones));

        let due = series
            .filter(|_| set.start.is_none() && set.rules.is_empty())
            .and_then(|series| Placed::of_due(series, zones));

        Self {
            set,
            zone,
            floating,
            date,
            span,
            periods,
            overrides,
            due,
        }
    }

    /// The recurrence set the timeline walks, on the series' clock.
    pub fn set(&self) -> &IcalRecurSet {
        &self.set
    }

    /// Walk the occurrences overlapping the window `[from, until)`, in seconds
    /// since the Unix epoch, lazily and in identity order.
    ///
    /// An occurrence overlaps when it starts before `until` and ends after
    /// `from`, or, lasting no time, starts within the window. The walk skips
    /// ahead to the window rather than stepping from `DTSTART`, and an
    /// override moving an instance into the window from outside it is
    /// yielded too. A consumer bounding what one series yields takes from it.
    pub fn expand(&self, from: i64, until: i64) -> IcalRecurTimelineExpand<'_> {
        let reach = self
            .periods
            .iter()
            .map(|(_, span)| *span)
            .fold(self.span.reach(), i64::max);

        let shifts = self
            .set
            .overrides
            .iter()
            .filter(|over| over.this_and_future)
            .map(|over| over.start.seconds() - over.id.seconds());

        let (ahead, behind) = shifts.fold((0, 0), |(ahead, behind), shift| {
            (shift.max(ahead), shift.min(behind))
        });

        let bound = |seconds: i64| IcalRecurDateTime::from_seconds(seconds.clamp(EARLIEST, LATEST));

        let lo = bound(
            from.saturating_sub(SLACK)
                .saturating_sub(reach)
                .saturating_sub(ahead),
        );
        let hi = bound(until.saturating_add(SLACK).saturating_sub(behind));

        let mut walk = match &self.zone {
            Some(zone) => self.set.expand_in_zone(zone),
            None => self.set.expand(),
        };
        walk.skip_to(lo);

        // NOTE: An override whose identity the bounded walk never reaches may
        // still move its instance into the window, so those are placed apart,
        // before the walk and after it, keeping identity order.
        let mut before = VecDeque::new();
        let mut after = VecDeque::new();

        for (index, placed) in self.overrides.iter().enumerate() {
            let outside = match placed.id {
                id if id < lo => &mut before,
                id if id >= hi => &mut after,
                _ => continue,
            };

            if self.yields(placed.id) {
                outside.push_back(index);
            }
        }

        IcalRecurTimelineExpand {
            timeline: self,
            clock: Clock::of_zone(self.zone.clone()),
            walk,
            from,
            until,
            hi,
            before,
            after,
            due: self.due.is_some(),
            walking: true,
        }
    }

    /// Whether the full walk yields an identity, which an override's is unless
    /// an `EXDATE` or an `EXRULE` removes it.
    fn yields(&self, id: IcalRecurDateTime) -> bool {
        let mut walk = self.set.expand();
        walk.skip_to(id);
        walk.next().is_some_and(|occurrence| occurrence.id == id)
    }
}

/// The lazy walk of an [`IcalRecurTimeline`] over a window, in identity order.
pub struct IcalRecurTimelineExpand<'a> {
    timeline: &'a IcalRecurTimeline,
    clock: Clock,
    walk: IcalRecurSetExpand<'a>,
    from: i64,
    until: i64,
    hi: IcalRecurDateTime,
    /// The overrides placed before the walk and after it, as indices into
    /// the timeline's, in identity order.
    before: VecDeque<usize>,
    after: VecDeque<usize>,
    due: bool,
    walking: bool,
}

impl Iterator for IcalRecurTimelineExpand<'_> {
    type Item = IcalRecurInstant;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let instant = self.next_placed()?;

            let overlaps = instant.start < self.until
                && (instant.end > self.from
                    || (instant.end == instant.start && instant.start >= self.from));

            if overlaps {
                return Some(instant);
            }
        }
    }
}

impl IcalRecurTimelineExpand<'_> {
    /// The next occurrence placed, in the window or not.
    fn next_placed(&mut self) -> Option<IcalRecurInstant> {
        let timeline = self.timeline;

        if self.due {
            self.due = false;

            if let Some(placed) = &timeline.due {
                return Some(placed.instant(placed.start, false));
            }
        }

        if let Some(index) = self.before.pop_front() {
            return Some(self.of_override(index));
        }

        if self.walking {
            match self.walk.next() {
                Some(occurrence) if occurrence.id < self.hi => {
                    if let Some(index) = occurrence.over {
                        return Some(self.of_override(index));
                    }

                    let start = self.clock.instant(occurrence.start);
                    let id_instant = self.clock.instant(occurrence.id);

                    let end = match timeline
                        .periods
                        .binary_search_by_key(&occurrence.id, |(id, _)| *id)
                    {
                        Ok(index) => start + timeline.periods[index].1,
                        Err(_) => timeline.span.end(occurrence.start, start, &mut self.clock),
                    };

                    return Some(IcalRecurInstant {
                        id: occurrence.id,
                        id_instant,
                        start,
                        end: end.max(start),
                        over: false,
                        floating: timeline.floating,
                        date: timeline.date,
                    });
                }
                _ => self.walking = false,
            }
        }

        if let Some(index) = self.after.pop_front() {
            return Some(self.of_override(index));
        }

        None
    }

    /// The occurrence an override places, its identity resolved on the series'
    /// clock.
    fn of_override(&mut self, index: usize) -> IcalRecurInstant {
        let placed = &self.timeline.overrides[index];
        placed.instant(self.clock.instant(placed.id), true)
    }
}

/// An occurrence placed by a component of its own, an override or a task's
/// `DUE`, whose times do not depend on the window.
#[derive(Clone, Copy, Debug)]
struct Placed {
    id: IcalRecurDateTime,
    start: i64,
    end: i64,
    floating: bool,
    date: bool,
}

impl Placed {
    /// The occurrence an override component places, at its own `DTSTART`
    /// read on its own clock and ending as its own component says.
    fn of_component(
        component: &IcalComponent<'_>,
        id: IcalRecurDateTime,
        zones: &[IcalTz],
    ) -> Option<Self> {
        let (local, mut clock) = written(prop_of(component, IcalPropKind::DtStart)?, zones)?;
        let start = clock.instant(local);
        let span = Span::of_component(component, zones);

        Some(Self {
            id,
            start,
            end: span.end(local, start, &mut clock).max(start),
            floating: clock.floating(),
            date: prop_of(component, IcalPropKind::DtStart).is_some_and(is_date),
        })
    }

    /// The one occurrence a task with a `DUE` and no `DTSTART` is, lasting no
    /// time.
    fn of_due(component: &IcalComponent<'_>, zones: &[IcalTz]) -> Option<Self> {
        if !is_kind(component, IcalComponentKind::VTodo) {
            return None;
        }

        let due = prop_of(component, IcalPropKind::Due)?;
        let (local, mut clock) = written(due, zones)?;
        let start = clock.instant(local);

        Some(Self {
            id: local,
            start,
            end: start,
            floating: clock.floating(),
            date: is_date(due),
        })
    }

    fn instant(&self, id_instant: i64, over: bool) -> IcalRecurInstant {
        IcalRecurInstant {
            id: self.id,
            id_instant,
            start: self.start,
            end: self.end,
            over,
            floating: self.floating,
            date: self.date,
        }
    }
}

/// How an end is placed from its start.
#[derive(Clone, Copy, Debug)]
enum Span {
    /// A number of seconds on the timeline: a `DTEND` span, or a `DURATION`
    /// with no day in it.
    Exact(i64),
    /// Days on the start's wall clock, then seconds on the timeline
    /// (RFC 5545 3.3.6).
    Nominal { days: i64, seconds: i64 },
}

impl Span {
    /// How a component's occurrences end: its `DTEND` (a `VTODO`'s `DUE`)
    /// against its `DTSTART`, each on its own clock, else its `DURATION`, else
    /// RFC 5545 3.6.1's default.
    fn of_component(component: &IcalComponent<'_>, zones: &[IcalTz]) -> Self {
        let start = prop_of(component, IcalPropKind::DtStart);

        let end = match is_kind(component, IcalComponentKind::VTodo) {
            true => IcalPropKind::Due,
            false => IcalPropKind::DtEnd,
        };

        let instant = |prop| {
            let (local, mut clock) = written(prop, zones)?;
            Some(clock.instant(local))
        };

        if let Some(start) = start.and_then(instant)
            && let Some(end) = prop_of(component, end).and_then(instant)
        {
            return Self::Exact(end - start);
        }

        let duration =
            prop_of(component, IcalPropKind::Duration).and_then(|prop| match &prop.value {
                IcalValue::Duration(duration) => duration.nominal(),
                _ => None,
            });

        match duration {
            Some((0, seconds)) => Self::Exact(seconds),
            Some((days, seconds)) => Self::Nominal { days, seconds },
            None if start.is_some_and(is_date) && is_kind(component, IcalComponentKind::VEvent) => {
                Self::Nominal {
                    days: 1,
                    seconds: 0,
                }
            }
            None => Self::Exact(0),
        }
    }

    /// The end of an occurrence starting at a civil time on `clock`, which is
    /// the instant `start`.
    fn end(&self, local: IcalRecurDateTime, start: i64, clock: &mut Clock) -> i64 {
        match *self {
            Self::Exact(seconds) => start.saturating_add(seconds),
            Self::Nominal { days, seconds } => {
                let local = local
                    .seconds()
                    .saturating_add(days.saturating_mul(DAY))
                    .clamp(EARLIEST, LATEST);

                clock
                    .instant(IcalRecurDateTime::from_seconds(local))
                    .saturating_add(seconds)
            }
        }
    }

    /// The most an occurrence can last, in seconds, a day of slack covering
    /// what a transition adds to a nominal one.
    fn reach(&self) -> i64 {
        match *self {
            Self::Exact(seconds) => seconds.max(0),
            Self::Nominal { days, seconds } => days
                .saturating_add(1)
                .saturating_mul(DAY)
                .saturating_add(seconds)
                .max(0),
        }
    }
}

/// The clock a civil time is resolved on: a zone, UTC, or none.
#[derive(Clone, Debug)]
enum Clock {
    Utc,
    Floating,
    Zone(IcalTzTransitions),
}

impl Clock {
    /// The clock a written time is told on, a `TZID` no zone defines reading
    /// floating.
    fn of(zone: &IcalRecurZone, zones: &[IcalTz]) -> Self {
        match zone {
            IcalRecurZone::Utc => Self::Utc,
            IcalRecurZone::Floating => Self::Floating,
            IcalRecurZone::Tz(id) => {
                Self::of_zone(zones.iter().find(|zone| zone.id == *id).cloned())
            }
        }
    }

    fn of_zone(zone: Option<IcalTz>) -> Self {
        match zone {
            Some(zone) => Self::Zone(IcalTzTransitions::of_zone(zone)),
            None => Self::Floating,
        }
    }

    /// The instant a civil time names on this clock, read as a written
    /// `DATE-TIME` (RFC 5545 3.3.5): a gap takes the offset before it, a fold
    /// the earlier one, and no zone reads as UTC.
    fn instant(&mut self, local: IcalRecurDateTime) -> i64 {
        match self {
            Self::Zone(zone) => zone.resolve(local).literal_instant(local),
            Self::Utc | Self::Floating => local.seconds(),
        }
    }

    fn floating(&self) -> bool {
        matches!(self, Self::Floating)
    }
}

/// The civil time a date property states, and the clock it is told on.
fn written(prop: &IcalProp<'_>, zones: &[IcalTz]) -> Option<(IcalRecurDateTime, Clock)> {
    let text = text_of(&prop.value)?;
    let local = IcalRecurDateTime::parse(text).ok()?;

    Some((
        local,
        Clock::of(&IcalRecurZone::of(&prop.params, text), zones),
    ))
}

/// The identity of every `RDATE` period of a component, told on the series'
/// clock, and the seconds it lasts: to its end on its own clock, or for its
/// duration from its start.
fn periods_of(
    component: &IcalComponent<'_>,
    series: &IcalRecurZone,
    zones: &[IcalTz],
) -> Vec<(IcalRecurDateTime, i64)> {
    let mut periods = Vec::new();

    for prop in &component.props {
        if !matches!(prop.name, IcalPropName::Kind(IcalPropKind::RDate)) {
            continue;
        }

        let items = match &prop.value {
            IcalValue::DateTimeList(dates) => dates.0.iter().map(|item| item.as_ref()).collect(),
            IcalValue::Period(period) => vec![period.0.as_ref()],
            _ => Vec::new(),
        };

        for item in items {
            let Some((start, end)) = item.split_once('/') else {
                continue;
            };

            let Ok(local) = IcalRecurDateTime::parse(start) else {
                continue;
            };

            let zone = IcalRecurZone::of(&prop.params, start);
            let mut clock = Clock::of(&zone, zones);
            let at = clock.instant(local);

            let until = match IcalRecurDateTime::parse(end) {
                Ok(end_local) => {
                    Clock::of(&IcalRecurZone::of(&prop.params, end), zones).instant(end_local)
                }
                Err(_) => {
                    let duration = IcalDuration::from(end);

                    let Some((days, seconds)) = duration.nominal() else {
                        continue;
                    };

                    Span::Nominal { days, seconds }.end(local, at, &mut clock)
                }
            };

            periods.push((zone.tell(local, series, zones), (until - at).max(0)));
        }
    }

    periods.sort_by_key(|(id, _)| *id);
    periods.dedup_by_key(|(id, _)| *id);
    periods
}

/// The first property of a kind a component carries.
fn prop_of<'a, 'v>(
    component: &'a IcalComponent<'v>,
    kind: IcalPropKind,
) -> Option<&'a IcalProp<'v>> {
    component
        .props
        .iter()
        .find(|prop| matches!(prop.name, IcalPropName::Kind(found) if found == kind))
}

/// Whether a date property states a date rather than a date-time.
fn is_date(prop: &IcalProp<'_>) -> bool {
    match &prop.value {
        IcalValue::Date(_) => true,
        value => text_of(value).is_some_and(|text| text.len() == 8),
    }
}

/// Whether a component is of a kind.
fn is_kind(component: &IcalComponent<'_>, kind: IcalComponentKind) -> bool {
    matches!(component.name, IcalComponentName::Kind(found) if found == kind)
}
