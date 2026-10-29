//! # When an alarm fires
//!
//! The instants the `VALARM`s of one event or to-do fire at (RFC 5545 3.6.6,
//! 3.8.6.3), over a window of instants.
//!
//! A `VALARM` has no time of its own: a relative `TRIGGER` counts from its
//! parent's start or end, so it fires once per occurrence of a recurring
//! parent, and an absolute one fires once at the UTC instant it names. This
//! module is the step after [`recur::instant`](crate::recur::instant): the
//! timeline places every occurrence, then each alarm of the component that
//! placed it is counted from there.
//!
//! [`IcalAlarms::of_uid`] reads one `UID` of a calendar, series and overrides,
//! through the zones it is given, and [`IcalAlarms::firings`] lists the
//! firings over a window, in time order.
//!
//! ## Anchors
//!
//! A `VEVENT` starts at its `DTSTART` and ends at its `DTEND`, else after its
//! `DURATION`, else at RFC 5545 3.6.1's default. A `VTODO` starts at its
//! `DTSTART` and ends at its `DUE`, else after its `DURATION`. A trigger
//! counting from an anchor its parent lacks fires nothing and is reported
//! ([`IcalAlarms::problems`]), save one leniency: a trigger counting from the
//! start of a to-do with a `DUE` and no `DTSTART` counts from its `DUE`, since
//! clients write it to mean "before it is due".
//!
//! ## Durations
//!
//! A trigger's weeks and days are added on the anchor's wall clock, its hours,
//! minutes and seconds on the timeline (RFC 5545 3.3.6). A floating anchor is
//! read in the zone the caller supplies, else as if on UTC.
//!
//! ## What is not filtered
//!
//! Nothing: an alarm `ACKNOWLEDGED` (RFC 9074 6) is carried on its firings,
//! and the parent's `STATUS` is not looked at. What to skip is the consumer's
//! choice.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    param::IcalParam,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    recur::{
        IcalRecurDateTime,
        instant::{IcalRecurInstant, IcalRecurTimeline},
        set::{IcalRecurOverride, IcalRecurZone, has, text_of, uid_of},
    },
    tz::IcalTz,
    value::IcalValue,
};

/// A day, in seconds.
const DAY: i64 = 86_400;

/// The widening of a walk beyond the furthest trigger, covering what a
/// transition adds to a nominal day.
const SLACK: i64 = 2 * DAY;

/// One firing of an alarm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalAlarmFiring {
    /// The index of the alarm among the `VALARM`s of the component holding
    /// it: the series, or the override [`over`](Self::over) names.
    pub alarm: usize,
    /// The alarm's own `UID` (RFC 9074 4), when it has one.
    pub uid: Option<String>,
    /// The identity of the override holding the alarm, from its
    /// `RECURRENCE-ID`, on the series' clock; `None` for the series' own.
    pub over: Option<IcalRecurDateTime>,
    /// The identity of the occurrence the firing belongs to, on the series'
    /// clock; `None` for an item that does not recur, and for an absolute
    /// trigger, which belongs to none.
    pub occurrence: Option<IcalRecurDateTime>,
    /// The rank of the firing in its alarm's repetition, zero for the trigger
    /// itself (RFC 5545 3.8.6.2).
    pub repeat: u32,
    /// The alarm's `ACTION`, as written.
    pub action: Option<String>,
    /// When it fires, in seconds since the Unix epoch.
    pub instant: i64,
    /// When the alarm was last dismissed, from its `ACKNOWLEDGED`, in seconds
    /// since the Unix epoch.
    pub acknowledged: Option<i64>,
}

/// An alarm that fires nothing, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalAlarmProblem {
    /// The index of the alarm among the `VALARM`s of the component holding
    /// it.
    pub alarm: usize,
    /// The identity of the override holding it; `None` for the series.
    pub over: Option<IcalRecurDateTime>,
    /// What is wrong.
    pub kind: IcalAlarmProblemKind,
}

/// What keeps an alarm from firing, or from firing as written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalAlarmProblemKind {
    /// No `TRIGGER`, or one no reading makes sense of.
    NoTrigger,
    /// A trigger counting from the start of a parent that has none.
    NoStart,
    /// A trigger counting from the end of a parent that has none.
    NoEnd,
    /// A `REPEAT` without a `DURATION` or the reverse: the alarm fires once.
    LoneRepeat,
}

/// The alarms of one `UID` of a calendar, ready to be resolved over windows
/// of instants.
///
/// Holds what it needs and nothing borrowed, so it outlives the calendar it
/// was read from.
#[derive(Clone, Debug)]
pub struct IcalAlarms {
    timeline: IcalRecurTimeline,
    zones: Vec<IcalTz>,
    /// Whether the series does not recur.
    single: bool,
    series: Holder,
    /// Every override holding alarms or not, in order of identity.
    overrides: Vec<Holder>,
    problems: Vec<IcalAlarmProblem>,
}

impl IcalAlarms {
    /// Read the alarms of one `UID` among a calendar's components, every time
    /// resolved through `zones`, the first zone of a name winning.
    pub fn of_uid(components: &[IcalComponent<'_>], uid: &str, zones: &[IcalTz]) -> Self {
        let timeline = IcalRecurTimeline::of_uid(components, uid, zones);
        let set = timeline.set();

        let ours = || {
            components
                .iter()
                .filter(move |component| uid_of(component) == Some(uid))
        };

        let mut problems = Vec::new();

        let series = ours()
            .rfind(|component| !has(component, IcalPropKind::RecurrenceId))
            .map(|component| Holder::of_component(component, None, &mut problems))
            .unwrap_or_default();

        let mut overrides: Vec<Holder> = ours()
            .filter(|component| has(component, IcalPropKind::RecurrenceId))
            .filter_map(|component| {
                let over = IcalRecurOverride::of_component(component, &set.zone, zones)?;
                let mut holder = Holder::of_component(component, Some(over.id), &mut problems);
                holder.this_and_future = over.this_and_future;
                Some(holder)
            })
            .collect();

        // NOTE: Stable, so of two overrides naming one instance the first
        // listed wins, as it does on the timeline.
        overrides.sort_by_key(|holder| holder.id);

        let single = set.rules.is_empty() && set.dates.is_empty() && overrides.is_empty();

        Self {
            single,
            timeline,
            zones: zones.to_vec(),
            series,
            overrides,
            problems,
        }
    }

    /// The alarms that fire nothing, or not as written, in the order the
    /// components list them.
    pub fn problems(&self) -> &[IcalAlarmProblem] {
        &self.problems
    }

    /// Every firing whose instant falls in the window `[from, until)`, in
    /// seconds since the Unix epoch, in time order.
    ///
    /// A floating anchor resolves in `floating` when given, else as if on
    /// UTC. The list is built whole: a window is what bounds it.
    pub fn firings(
        &self,
        from: i64,
        until: i64,
        floating: Option<&IcalTz>,
    ) -> Vec<IcalAlarmFiring> {
        let mut firings = Vec::new();

        let holders = || core::iter::once(&self.series).chain(&self.overrides);

        for holder in holders() {
            for (index, alarm) in holder.alarms.iter().enumerate() {
                if let Trigger::Absolute(local, zone) = &alarm.trigger {
                    let clock = Clock::of(zone, &self.zones, floating);
                    let instant = clock.instant(*local);
                    alarm.fire(
                        index,
                        holder.id,
                        None,
                        instant,
                        &clock,
                        from,
                        until,
                        &mut firings,
                    );
                }
            }
        }

        let (behind, ahead) = holders()
            .flat_map(|holder| &holder.alarms)
            .filter_map(Alarm::reach)
            .fold((0, 0), |(behind, ahead), (lo, hi)| {
                (lo.min(behind), hi.max(ahead))
            });

        let walk_from = from.saturating_sub(ahead).saturating_sub(SLACK);
        let walk_until = until.saturating_sub(behind).saturating_add(SLACK);

        for instant in self.timeline.expand(walk_from, walk_until) {
            let holder = self.holder_of(&instant);
            let occurrence = (!self.single).then_some(instant.id);

            for (index, alarm) in holder.alarms.iter().enumerate() {
                let Trigger::Relative { end, days, seconds } = alarm.trigger else {
                    continue;
                };

                let anchor = match end {
                    true => holder.end.as_ref().map(|zone| (instant.end, zone)),
                    false => holder.start.as_ref().map(|zone| (instant.start, zone)),
                };

                let Some((at, zone)) = anchor else {
                    continue;
                };

                let clock = Clock::of(zone, &self.zones, floating);
                let at = clock.reread(at);
                let fires = clock.add(at, days, seconds);

                alarm.fire(
                    index,
                    holder.id,
                    occurrence,
                    fires,
                    &clock,
                    from,
                    until,
                    &mut firings,
                );
            }
        }

        firings.sort_by(|a, b| {
            (a.instant, a.occurrence, a.over, a.alarm, a.repeat).cmp(&(
                b.instant,
                b.occurrence,
                b.over,
                b.alarm,
                b.repeat,
            ))
        });

        firings
    }

    /// The component whose alarms an occurrence fires: the override placing
    /// it, else the latest `THISANDFUTURE` override before it, else the
    /// series.
    fn holder_of(&self, instant: &IcalRecurInstant) -> &Holder {
        if instant.over
            && let Some(holder) = self.overrides.iter().find(|h| h.id == Some(instant.id))
        {
            return holder;
        }

        self.overrides
            .iter()
            .rev()
            .find(|holder| holder.this_and_future && holder.id.is_some_and(|id| id < instant.id))
            .unwrap_or(&self.series)
    }
}

/// A component holding alarms: the series, or one override.
#[derive(Clone, Debug, Default)]
struct Holder {
    /// The override's identity, `None` for the series.
    id: Option<IcalRecurDateTime>,
    this_and_future: bool,
    /// The clock of the start anchor, `None` when the component has none.
    start: Option<IcalRecurZone>,
    /// The clock of the end anchor, `None` when the component has none.
    end: Option<IcalRecurZone>,
    alarms: Vec<Alarm>,
}

impl Holder {
    fn of_component(
        component: &IcalComponent<'_>,
        id: Option<IcalRecurDateTime>,
        problems: &mut Vec<IcalAlarmProblem>,
    ) -> Self {
        let todo = matches!(
            component.name,
            IcalComponentName::Kind(IcalComponentKind::VTodo)
        );

        let zone_of = |kind| prop_of(component, kind).and_then(zone_of);

        let dtstart = zone_of(IcalPropKind::DtStart);
        let duration = has(component, IcalPropKind::Duration);

        let (start, end) = match todo {
            true => {
                let due = zone_of(IcalPropKind::Due);
                let end = due.clone().or_else(|| dtstart.clone().filter(|_| duration));
                // NOTE: The leniency of a to-do due and not started: the
                // timeline places it at its due time, which is its start.
                let start = dtstart.or(due);
                (start, end)
            }
            false => {
                let end = zone_of(IcalPropKind::DtEnd).or_else(|| dtstart.clone());
                (dtstart, end)
            }
        };

        let mut alarms = Vec::new();

        let valarms = component.components.iter().filter(|component| {
            matches!(
                component.name,
                IcalComponentName::Kind(IcalComponentKind::VAlarm)
            )
        });

        for (index, valarm) in valarms.enumerate() {
            let mut problem = |kind| {
                problems.push(IcalAlarmProblem {
                    alarm: index,
                    over: id,
                    kind,
                })
            };

            let alarm = Alarm::of_component(valarm);

            match alarm.trigger {
                Trigger::None => problem(IcalAlarmProblemKind::NoTrigger),
                Trigger::Relative { end: false, .. } if start.is_none() => {
                    problem(IcalAlarmProblemKind::NoStart)
                }
                Trigger::Relative { end: true, .. } if end.is_none() => {
                    problem(IcalAlarmProblemKind::NoEnd)
                }
                _ => {}
            }

            if alarm.lone_repeat {
                problem(IcalAlarmProblemKind::LoneRepeat);
            }

            alarms.push(alarm);
        }

        Self {
            id,
            this_and_future: false,
            start,
            end,
            alarms,
        }
    }
}

/// One `VALARM`, read.
#[derive(Clone, Debug)]
struct Alarm {
    trigger: Trigger,
    /// How many more times it fires, and the nominal days and exact seconds
    /// between two firings.
    repeat: Option<(u32, i64, i64)>,
    lone_repeat: bool,
    uid: Option<String>,
    action: Option<String>,
    acknowledged: Option<i64>,
}

/// When an alarm fires.
#[derive(Clone, Debug)]
enum Trigger {
    /// Counted from the start, or the end, in nominal days and exact seconds.
    Relative {
        end: bool,
        days: i64,
        seconds: i64,
    },
    /// At a civil time on a clock, UTC unless written without its `Z`.
    Absolute(IcalRecurDateTime, IcalRecurZone),
    None,
}

impl Alarm {
    fn of_component(valarm: &IcalComponent<'_>) -> Self {
        let trigger = prop_of(valarm, IcalPropKind::Trigger).map_or(Trigger::None, |prop| {
            let end = prop.params.iter().any(|param| {
                matches!(param, IcalParam::Related(related) if related.eq_ignore_ascii_case("END"))
            });

            match &prop.value {
                IcalValue::Duration(duration) => match duration.nominal() {
                    Some((days, seconds)) => Trigger::Relative { end, days, seconds },
                    None => Trigger::None,
                },
                value => text_of(value)
                    .and_then(|text| {
                        let local = IcalRecurDateTime::parse(text).ok()?;
                        Some(Trigger::Absolute(
                            local,
                            IcalRecurZone::of(&prop.params, text),
                        ))
                    })
                    .unwrap_or(Trigger::None),
            }
        });

        let count = prop_of(valarm, IcalPropKind::Repeat).and_then(|prop| match &prop.value {
            IcalValue::Integer(integer) => integer.get(),
            _ => None,
        });

        let every = prop_of(valarm, IcalPropKind::Duration).and_then(|prop| match &prop.value {
            IcalValue::Duration(duration) => duration.nominal(),
            _ => None,
        });

        let (repeat, lone_repeat) = match (count, every) {
            (Some(count), Some((days, seconds))) => {
                let count = u32::try_from(count.max(0)).unwrap_or(u32::MAX);
                (Some((count, days, seconds)), false)
            }
            (None, None) => (None, false),
            _ => (None, true),
        };

        let text = |kind| {
            prop_of(valarm, kind).and_then(|prop| match &prop.value {
                IcalValue::Text(text) => Some(text.0.to_string()),
                IcalValue::Unknown(_) => None,
                value => text_of(value).map(ToString::to_string),
            })
        };

        let acknowledged = prop_of(valarm, IcalPropKind::Acknowledged)
            .and_then(|prop| text_of(&prop.value))
            .and_then(|text| IcalRecurDateTime::parse(text).ok())
            .map(|local| local.seconds());

        Self {
            trigger,
            repeat,
            lone_repeat,
            uid: text(IcalPropKind::Uid),
            action: text(IcalPropKind::Action),
            acknowledged,
        }
    }

    /// The span firings reach around their anchor, in seconds: the earliest
    /// and the latest offset, repetitions included, `None` for an alarm
    /// counted from no anchor.
    fn reach(&self) -> Option<(i64, i64)> {
        let Trigger::Relative { days, seconds, .. } = self.trigger else {
            return None;
        };

        let first = days.saturating_mul(DAY).saturating_add(seconds);

        let last = match self.repeat {
            Some((count, days, seconds)) => days
                .saturating_mul(DAY)
                .saturating_add(seconds)
                .saturating_mul(i64::from(count))
                .saturating_add(first),
            None => first,
        };

        Some((first.min(last), first.max(last)))
    }

    /// Push the firings of this alarm from its first instant, those in the
    /// window `[from, until)`, its repetitions counted on `clock`.
    #[allow(clippy::too_many_arguments)]
    fn fire(
        &self,
        index: usize,
        over: Option<IcalRecurDateTime>,
        occurrence: Option<IcalRecurDateTime>,
        first: i64,
        clock: &Clock<'_>,
        from: i64,
        until: i64,
        firings: &mut Vec<IcalAlarmFiring>,
    ) {
        let (count, days, seconds) = self.repeat.unwrap_or((0, 0, 0));
        let mut instant = first;

        for repeat in 0..=count {
            if repeat > 0 {
                instant = clock.add(instant, days, seconds);
            }

            if instant >= until && days >= 0 && seconds >= 0 {
                break;
            }

            if instant < from || instant >= until {
                continue;
            }

            firings.push(IcalAlarmFiring {
                alarm: index,
                uid: self.uid.clone(),
                over,
                occurrence,
                repeat,
                action: self.action.clone(),
                instant,
                acknowledged: self.acknowledged,
            });
        }
    }
}

/// The wall clock an anchor is told on, a floating one read in the zone the
/// caller supplies.
enum Clock<'a> {
    Utc,
    /// A floating clock with no zone supplied, read as if on UTC.
    Floating,
    /// A floating clock read in the supplied zone.
    Supplied(&'a IcalTz),
    Zone(&'a IcalTz),
}

impl<'a> Clock<'a> {
    fn of(zone: &IcalRecurZone, zones: &'a [IcalTz], floating: Option<&'a IcalTz>) -> Self {
        let floating = || floating.map_or(Self::Floating, Self::Supplied);

        match zone {
            IcalRecurZone::Utc => Self::Utc,
            IcalRecurZone::Floating => floating(),
            IcalRecurZone::Tz(id) => match zones.iter().find(|zone| zone.id == *id) {
                Some(zone) => Self::Zone(zone),
                None => floating(),
            },
        }
    }

    /// The instant a written civil time names on this clock (RFC 5545 3.3.5).
    fn instant(&self, local: IcalRecurDateTime) -> i64 {
        match self {
            Self::Utc | Self::Floating => local.seconds(),
            Self::Supplied(zone) | Self::Zone(zone) => zone.resolve(local).literal_instant(local),
        }
    }

    /// The instant an anchor the timeline placed names on this clock: the
    /// timeline reads a floating time as if on UTC, so a supplied zone reads
    /// it again.
    fn reread(&self, at: i64) -> i64 {
        match self {
            Self::Supplied(_) => self.instant(IcalRecurDateTime::from_seconds(at)),
            _ => at,
        }
    }

    /// The local time this clock shows at an instant.
    fn local(&self, instant: i64) -> IcalRecurDateTime {
        match self {
            Self::Utc | Self::Floating => IcalRecurDateTime::from_seconds(instant),
            Self::Supplied(zone) | Self::Zone(zone) => zone.local(instant),
        }
    }

    /// An instant moved by nominal days on this clock, then by exact seconds
    /// (RFC 5545 3.3.6).
    fn add(&self, instant: i64, days: i64, seconds: i64) -> i64 {
        let moved = match days {
            0 => instant,
            days => {
                let local = self
                    .local(instant)
                    .seconds()
                    .saturating_add(days.saturating_mul(DAY));
                self.instant(IcalRecurDateTime::from_seconds(local))
            }
        };

        moved.saturating_add(seconds)
    }
}

/// The clock a date property's value is told on.
fn zone_of(prop: &IcalProp<'_>) -> Option<IcalRecurZone> {
    let text = text_of(&prop.value)?;
    IcalRecurDateTime::parse(text).ok()?;
    Some(IcalRecurZone::of(&prop.params, text))
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
