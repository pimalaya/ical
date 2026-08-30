//! # Time zones
//!
//! Turning a civil date-time into a UTC offset, using only the `VTIMEZONE` that
//! travels inside the calendar.
//!
//! Expansion is civil by design ([`recur`](crate::recur)) and that boundary
//! stays: nothing here changes what a rule denotes. What it adds is the step
//! after, the one nobody could take before.
//!
//! A caller holding an occurrence and the calendar it came from can ask what
//! offset was in force, with no time-zone database and no new dependency,
//! because RFC 5545 3.6.5 makes a calendar carry its own rules.
//!
//! A `VTIMEZONE` is a list of observances, each with the offset before it,
//! the offset after it, and a recurrence rule saying when it takes effect.
//! "Observance" is RFC 5545 3.6.5's own word for one such rule, reused by RFC
//! 7808; a datetime library would call the instants it generates transitions,
//! which is what [`IcalTzTransition`] here is.
//!
//! ## The two hard cases are answered, not guessed
//!
//! A local clock is not a bijection. When it springs forward, the times it
//! jumps over never happen; when it falls back, the times it repeats happen
//! twice.
//!
//! [`resolve`](IcalTz::resolve) reports both as what they are, with the
//! offsets either side, rather than picking one and calling it the answer.
//! Choosing belongs to the caller, who knows whether a skipped alarm should
//! fire early, late or not at all. [`instant`](IcalTzOffset::instant) is the
//! one place a choice is made, and it says which of its three answers the RFC
//! settled and which it did not.
//!
//! The gap has one caller that does not choose: a recurrence rule generating
//! an instance in one is generating something that never happens, which RFC
//! 5545 3.3.10 drops from the set outright. That filter reads the zone through
//! [`IcalTzTransitions`], since it asks once per candidate rather than once.
//!
//! ## What is read, and what is not
//!
//! An observance contributes its `DTSTART`, its `RRULE`s and its `RDATE`s
//! through the same [`IcalRecurSet`] every other component uses, so the
//! transitions of a zone are just another recurrence set.
//!
//! `TZNAME`, `TZURL` and `LAST-MODIFIED` are not read: they name a zone, they
//! do not place it.
//!
//! Every `DTSTART` inside an observance is local to the offset *before* the
//! transition (RFC 5545 3.6.5), which is what makes a transition expressible
//! in two local times at once, and what the gap and the fold are made of.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    ical::Ical,
    prop::{IcalPropKind, IcalPropName},
    recur::{IcalRecurDateTime, set::IcalRecurSet},
    value::IcalValue,
};

/// What offset is in force at one civil local time, or why no single one is.
///
/// The answer to a resolution rather than an offset value: a local clock is
/// not a bijection, so a time may have one offset, none, or two. The wire
/// spelling of an offset is [`IcalUtcOffset`].
///
/// [`IcalUtcOffset`]: crate::value::utc_offset::IcalUtcOffset
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalTzOffset {
    /// The local time is unambiguous, and this offset is in force. Seconds east
    /// of UTC, so `-0500` is `-18000`.
    One(i32),
    /// The local time never happens: a transition jumped over it.
    ///
    /// An instance a recurrence rule generates here is dropped from the set
    /// and costs no `COUNT` slot, by RFC 5545 3.3.10 rather than by any choice
    /// of the caller's. See [`recur::expand`](crate::recur::expand).
    Gap {
        /// The offset in force before the transition.
        before: i32,
        /// The offset in force after it.
        after: i32,
    },
    /// The local time happens twice.
    Fold {
        /// The offset of its first occurrence.
        earlier: i32,
        /// The offset of its second.
        later: i32,
    },
}

impl IcalTzOffset {
    /// The offset, when the local time has exactly one. `None` for a gap or a
    /// fold, which is the whole point of distinguishing them.
    pub fn unambiguous(&self) -> Option<i32> {
        match self {
            Self::One(offset) => Some(*offset),
            _ => None,
        }
    }

    /// The instant a civil local time names under this resolution, in seconds
    /// since the Unix epoch.
    ///
    /// The crossing, named once. `None` for a gap is the specification's own
    /// answer (RFC 5545 3.3.10): a local time that never happens names no
    /// instant. The earlier of a fold's two is a default the RFC does not
    /// mandate, and a caller wanting the later one reads it off the variant.
    pub fn instant(&self, local: IcalRecurDateTime) -> Option<i64> {
        let offset = match self {
            Self::One(offset) => *offset,
            Self::Gap { .. } => return None,
            Self::Fold { earlier, .. } => *earlier,
        };

        Some(local.seconds() - i64::from(offset))
    }
}

/// One `STANDARD` or `DAYLIGHT` observance (RFC 5545 3.6.5): when it takes
/// effect, and the offsets either side of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalTzObservance {
    /// Whether this is a `DAYLIGHT` observance rather than a `STANDARD` one.
    pub daylight: bool,
    /// The offset in force before this observance takes effect
    /// (`TZOFFSETFROM`), in seconds east of UTC.
    pub from: i32,
    /// The offset in force after it (`TZOFFSETTO`), in seconds east of UTC.
    pub to: i32,
    /// When it takes effect, as a recurrence set. Every date in it is local to
    /// [`from`](Self::from).
    pub onsets: IcalRecurSet,
}

/// A `VTIMEZONE`, read into the observances that place it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalTz {
    /// The `TZID` this zone answers to, verbatim.
    pub id: String,
    /// Its observances, in source order.
    pub observances: Vec<IcalTzObservance>,
}

/// One transition of a zone: the instant it happens, expressed in the local
/// time before it, and the offsets either side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IcalTzTransition {
    /// The onset in the local time *before* the transition, as an observance's
    /// `DTSTART` spells it (RFC 5545 3.6.5).
    pub local: IcalRecurDateTime,
    /// The offset in force before it, in seconds east of UTC.
    pub from: i32,
    /// The offset in force after it.
    pub to: i32,
}

impl IcalTzTransition {
    /// The same instant in the local time *after* the transition.
    ///
    /// The pair of it and [`local`](Self::local) bounds what a spring-forward
    /// jumps over and what a fall-back repeats.
    pub fn after_local(&self) -> IcalRecurDateTime {
        IcalRecurDateTime::from_seconds(
            self.local.seconds() + i64::from(self.to) - i64::from(self.from),
        )
    }
}

/// A zone's transitions, materialised: resolution by lookup, not by expansion.
///
/// [`IcalTz::resolve`] re-expands every observance on every call, which is the
/// right shape for the one question a caller asks about an occurrence. Asking
/// once per candidate of a recurrence rule is a different shape, and this is
/// it: the transitions are held, and the held span doubles whenever a query
/// runs past it, so walking a rule forward costs one expansion amortised
/// rather than one per date.
#[derive(Clone, Debug)]
pub struct IcalTzTransitions {
    zone: IcalTz,
    transitions: Vec<IcalTzTransition>,
    /// The last year [`transitions`](Self::transitions) covers, before
    /// anything is materialised.
    through: Option<i32>,
    /// How many years past a query the next materialisation reaches.
    span: i32,
}

impl IcalTzTransitions {
    /// Hold a zone, materialising nothing until something is asked of it.
    pub fn of_zone(zone: IcalTz) -> Self {
        Self {
            zone,
            transitions: Vec::new(),
            through: None,
            span: 1,
        }
    }

    /// The offset in force at a civil local time, or the gap or fold it is in,
    /// exactly as [`IcalTz::resolve`] answers it.
    pub fn resolve(&mut self, local: IcalRecurDateTime) -> IcalTzOffset {
        if self.through.is_none_or(|through| local.year > through) {
            let through = local.year.saturating_add(self.span);

            self.transitions = self.zone.transitions(through);
            self.through = Some(through);
            self.span = self.span.saturating_mul(2);
        }

        IcalTz::offset(&self.transitions, local)
    }

    /// Whether a civil local time is one the zone jumps over.
    pub fn is_gap(&mut self, local: IcalRecurDateTime) -> bool {
        matches!(self.resolve(local), IcalTzOffset::Gap { .. })
    }
}

impl IcalTz {
    /// Read a `VTIMEZONE` component into its observances.
    ///
    /// `None` when the component is not a `VTIMEZONE` or carries no `TZID`: a
    /// zone nobody can name is a zone nobody can ask about.
    pub fn of_component(component: &IcalComponent<'_>) -> Option<Self> {
        if !matches!(
            component.name,
            IcalComponentName::Kind(IcalComponentKind::VTimezone)
        ) {
            return None;
        }

        let id = component
            .props
            .iter()
            .find_map(|prop| match (&prop.name, &prop.value) {
                (IcalPropName::Kind(IcalPropKind::TzId), IcalValue::Text(text)) => {
                    Some(text.0.to_string())
                }
                _ => None,
            })?;

        let observances = component
            .components
            .iter()
            .filter_map(IcalTzObservance::of_component)
            .collect();

        Some(Self { id, observances })
    }

    /// The zone a calendar defines under `tzid`, if it defines one.
    pub fn of_calendar(ical: &Ical<'_>, tzid: &str) -> Option<Self> {
        ical.components
            .iter()
            .filter_map(Self::of_component)
            .find(|zone| zone.id == tzid)
    }

    /// The offset in force at a civil local time, or the gap or fold it is in.
    ///
    /// A zone with no observance resolves everything to UTC, stating no offset
    /// to apply. A local time before the first transition takes the offset that
    /// transition says came before it, which is what `TZOFFSETFROM` is for.
    ///
    /// One call materialises what it needs and drops it. A caller asking many
    /// times, as a zoned expansion does, holds the transitions instead
    /// ([`IcalTzTransitions`]).
    pub fn resolve(&self, local: IcalRecurDateTime) -> IcalTzOffset {
        Self::offset(&self.transitions(local.year), local)
    }

    /// Whether a civil local time is one this zone jumps over, so that no
    /// instant answers to it (RFC 5545 3.3.10).
    pub fn is_gap(&self, local: IcalRecurDateTime) -> bool {
        matches!(self.resolve(local), IcalTzOffset::Gap { .. })
    }

    /// Every transition this zone states up to the end of a year, in
    /// chronological order.
    ///
    /// One onset past the bound is taken from each observance, which is what
    /// makes the list answer for the year itself: a fold reaches back from the
    /// transition that closes it, and a zone whose rules all start later still
    /// states the offset that came before them.
    pub fn transitions(&self, through: i32) -> Vec<IcalTzTransition> {
        let mut transitions = Vec::new();

        for observance in &self.observances {
            for onset in observance.onsets.expand() {
                let transition = IcalTzTransition {
                    local: onset.start,
                    from: observance.from,
                    to: observance.to,
                };

                transitions.push(transition);

                // NOTE: Onsets come out in order, so there is no reason to walk
                // an endless rule past the bound the caller asked for.
                if transition.local.year > through {
                    break;
                }
            }
        }

        transitions.sort_unstable_by_key(|transition| transition.local);
        transitions
    }

    /// The offset a materialised transition list puts in force at a local
    /// time, the one resolution both [`resolve`](Self::resolve) and
    /// [`IcalTzTransitions`] answer with.
    fn offset(transitions: &[IcalTzTransition], local: IcalRecurDateTime) -> IcalTzOffset {
        let index = transitions.partition_point(|transition| transition.local <= local);
        let previous = index.checked_sub(1).map(|index| transitions[index]);
        let next = transitions.get(index).copied();

        // NOTE: A local time the last transition jumped over never happened.
        if let Some(transition) = previous
            && transition.to > transition.from
            && local < transition.after_local()
        {
            return IcalTzOffset::Gap {
                before: transition.from,
                after: transition.to,
            };
        }

        // NOTE: A local time the next transition is about to repeat happens
        // twice.
        if let Some(transition) = next
            && transition.to < transition.from
            && local >= transition.after_local()
        {
            return IcalTzOffset::Fold {
                earlier: transition.from,
                later: transition.to,
            };
        }

        match (previous, transitions.first()) {
            (Some(transition), _) => IcalTzOffset::One(transition.to),
            (None, Some(transition)) => IcalTzOffset::One(transition.from),
            (None, None) => IcalTzOffset::One(0),
        }
    }
}

impl IcalTzObservance {
    /// Read a `STANDARD` or `DAYLIGHT` component into an observance. `None` for
    /// anything else, and for one that states no offsets.
    pub fn of_component(component: &IcalComponent<'_>) -> Option<Self> {
        let daylight = match component.name {
            IcalComponentName::Kind(IcalComponentKind::Daylight) => true,
            IcalComponentName::Kind(IcalComponentKind::Standard) => false,
            _ => return None,
        };

        let mut from = None;
        let mut to = None;

        for prop in &component.props {
            let IcalPropName::Kind(kind) = prop.name else {
                continue;
            };

            let IcalValue::UtcOffset(offset) = &prop.value else {
                continue;
            };

            match kind {
                IcalPropKind::TzOffsetFrom => from = offset.seconds(),
                IcalPropKind::TzOffsetTo => to = offset.seconds(),
                _ => {}
            }
        }

        Some(Self {
            daylight,
            from: from?,
            to: to?,
            onsets: IcalRecurSet::of_component(component),
        })
    }
}
