//! # Time zones
//!
//! Turning a civil date-time into a UTC offset, using only the `VTIMEZONE` that
//! travels inside the calendar.
//!
//! Expansion is civil by design ([`recur`](crate::recur)) and that boundary
//! stays: nothing here changes what a rule denotes. What it adds is the step
//! after it, the one nobody could take before. A caller holding an occurrence
//! and the calendar it came from can now ask what offset was in force, with no
//! time-zone database and no new dependency, because RFC 5545 3.6.5 makes a
//! calendar carry its own rules: a `VTIMEZONE` is a list of observances, each
//! with the offset before it, the offset after it, and a recurrence rule saying
//! when it takes effect.
//!
//! ## The two hard cases are answered, not guessed
//!
//! A local clock is not a bijection. When it springs forward, the times it
//! jumps over never happen; when it falls back, the times it repeats happen
//! twice. [`resolve`](IcalTimezone::resolve) reports both as what they are,
//! with the offsets either side, rather than picking one and calling it the
//! answer. Choosing belongs to the caller, who knows whether a skipped alarm
//! should fire early, late or not at all.
//!
//! ## What is read, and what is not
//!
//! An observance contributes its `DTSTART`, its `RRULE`s and its `RDATE`s
//! through the same [`IcalRecurSet`] every other component uses, so the
//! transitions of a zone are just another recurrence set. `TZNAME`, `TZURL` and
//! `LAST-MODIFIED` are not read: they name a zone, they do not place it.
//!
//! Every `DTSTART` inside an observance is local to the offset *before* the
//! transition (RFC 5545 3.6.5), which is what makes a transition expressible in
//! two local times at once, and what the gap and the fold are made of.

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

/// What offset is in force at one civil local time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalOffset {
    /// The local time is unambiguous, and this offset is in force. Seconds east
    /// of UTC, so `-0500` is `-18000`.
    One(i32),
    /// The local time never happens: a transition jumped over it.
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

impl IcalOffset {
    /// The offset, when the local time has exactly one. `None` for a gap or a
    /// fold, which is the whole point of distinguishing them.
    pub fn unambiguous(&self) -> Option<i32> {
        match self {
            Self::One(offset) => Some(*offset),
            _ => None,
        }
    }
}

/// One `STANDARD` or `DAYLIGHT` observance: when it takes effect, and the
/// offsets either side of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalObservance {
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
pub struct IcalTimezone {
    /// The `TZID` this zone answers to, verbatim.
    pub id: String,
    /// Its observances, in source order.
    pub observances: Vec<IcalObservance>,
}

/// One transition of a zone: the instant it happens, expressed in the local
/// time before it, and the offsets either side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Transition {
    /// The onset in the local time *before* the transition, as `DTSTART`
    /// spells it.
    before_local: IcalRecurDateTime,
    from: i32,
    to: i32,
}

impl Transition {
    /// The same instant in the local time *after* the transition.
    fn after_local(&self) -> IcalRecurDateTime {
        IcalRecurDateTime::from_seconds(
            self.before_local.seconds() + i64::from(self.to) - i64::from(self.from),
        )
    }
}

impl IcalTimezone {
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
            .filter_map(IcalObservance::of_component)
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

    /// The offset in force at a civil local time, or the gap or fold it falls
    /// in.
    ///
    /// A zone with no observance at all resolves everything to UTC, since it
    /// states no offset to apply. A local time before the first transition
    /// takes the offset that transition says came before it, which is what
    /// `TZOFFSETFROM` is for.
    pub fn resolve(&self, local: IcalRecurDateTime) -> IcalOffset {
        let mut previous: Option<Transition> = None;
        let mut next: Option<Transition> = None;
        let mut first: Option<Transition> = None;

        for observance in &self.observances {
            for onset in observance.onsets.expand() {
                let transition = Transition {
                    before_local: onset.start,
                    from: observance.from,
                    to: observance.to,
                };

                if first.is_none_or(|held| transition.before_local < held.before_local) {
                    first = Some(transition);
                }

                if transition.before_local <= local {
                    if previous.is_none_or(|held| held.before_local < transition.before_local) {
                        previous = Some(transition);
                    }
                } else {
                    // NOTE: Onsets come out in order, so the first one past the
                    // query is this observance's only candidate for the next
                    // transition, and there is no reason to walk an endless
                    // rule any further.
                    if next.is_none_or(|held| transition.before_local < held.before_local) {
                        next = Some(transition);
                    }
                    break;
                }
            }
        }

        // NOTE: A local time the last transition jumped over never happened.
        if let Some(transition) = previous
            && transition.to > transition.from
            && local < transition.after_local()
        {
            return IcalOffset::Gap {
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
            return IcalOffset::Fold {
                earlier: transition.from,
                later: transition.to,
            };
        }

        match (previous, first) {
            (Some(transition), _) => IcalOffset::One(transition.to),
            (None, Some(transition)) => IcalOffset::One(transition.from),
            (None, None) => IcalOffset::One(0),
        }
    }
}

impl IcalObservance {
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
                IcalPropKind::TzOffsetFrom => from = parse_offset(&offset.0),
                IcalPropKind::TzOffsetTo => to = parse_offset(&offset.0),
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

/// Parse the RFC 5545 3.3.14 `+/-hhmm[ss]` form into seconds east of UTC.
fn parse_offset(text: &str) -> Option<i32> {
    let (sign, digits) = match text.as_bytes().first()? {
        b'+' => (1, &text[1..]),
        b'-' => (-1, &text[1..]),
        _ => (1, text),
    };

    if !matches!(digits.len(), 4 | 6) || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let part = |range: core::ops::Range<usize>| digits[range].parse::<i32>().ok();

    let hours = part(0..2)?;
    let minutes = part(2..4)?;
    let seconds = if digits.len() == 6 { part(4..6)? } else { 0 };

    Some(sign * (hours * 3600 + minutes * 60 + seconds))
}

#[cfg(test)]
mod tests {
    use crate::timezone::parse_offset;

    #[test]
    fn reads_every_offset_spelling() {
        assert_eq!(parse_offset("-0500"), Some(-18_000));
        assert_eq!(parse_offset("+0100"), Some(3_600));
        assert_eq!(parse_offset("+053045"), Some(19_845));
        assert_eq!(parse_offset("0000"), Some(0));
    }

    #[test]
    fn refuses_what_is_not_an_offset() {
        assert_eq!(parse_offset(""), None);
        assert_eq!(parse_offset("+5"), None);
        assert_eq!(parse_offset("+05:00"), None);
        assert_eq!(parse_offset("+0h00"), None);
    }
}
