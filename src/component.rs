//! # Components
//!
//! A decoded component and the iCalendar component-name vocabulary.
//!
//! iCalendar is a tree of components: a `VCALENDAR` holds `VEVENT`, `VTODO`,
//! `VJOURNAL`, `VFREEBUSY` and `VTIMEZONE` components; an event or to-do holds
//! `VALARM` components; a time zone holds `STANDARD` and `DAYLIGHT`
//! subcomponents; and (RFC 9073) a component may hold `PARTICIPANT`,
//! `VLOCATION` and `VRESOURCE` subcomponents. [`IcalComponent`] is that decoded
//! node: a name, its properties, and its nested components. The whole calendar
//! is the [`Ical`](crate::ical::Ical) aggregate, whose root is the `VCALENDAR`.
//!
//! A known name is held as the closed [`IcalComponentKind`] identity (its wire
//! spelling reached through `Deref` and `FromStr`); an unknown one keeps its
//! verbatim bytes. This module is pure model: no dependency on
//! [`crate::tree`].

use core::{error, fmt, ops, str};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::prop::IcalProp;

/// Parse iCalendar component kind error.
#[derive(Debug)]
pub struct ParseIcalComponentKindError(
    /// The iCalendar component that cannot be parsed.
    String,
);

impl fmt::Display for ParseIcalComponentKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse iCalendar component `{}`", self.0)
    }
}

impl error::Error for ParseIcalComponentKindError {}

/// A decoded component: its name, its properties, and its nested components.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalComponent<'a> {
    /// The component name (a known kind, or an unknown name kept verbatim).
    pub name: IcalComponentName<'a>,
    /// The properties of this component, in source order.
    pub props: Vec<IcalProp<'a>>,
    /// The components nested directly within this one, in source order.
    pub components: Vec<IcalComponent<'a>>,
}

/// A component name: a known iCalendar name, or an unknown one kept verbatim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalComponentName<'a> {
    /// A name in the closed iCalendar vocabulary.
    Kind(IcalComponentKind),
    /// Any other name, kept as written.
    Unknown(Cow<'a, str>),
}

impl ops::Deref for IcalComponentName<'_> {
    type Target = str;

    /// The name's wire string: the canonical spelling of a known name, or the
    /// verbatim text of an unknown one.
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Kind(kind) => kind,
            Self::Unknown(name) => name,
        }
    }
}

impl From<IcalComponentKind> for IcalComponentName<'_> {
    fn from(kind: IcalComponentKind) -> Self {
        Self::Kind(kind)
    }
}

impl<'a> From<Cow<'a, str>> for IcalComponentName<'a> {
    fn from(name: Cow<'a, str>) -> Self {
        match name.parse().ok() {
            Some(kind) => Self::Kind(kind),
            None => Self::Unknown(name),
        }
    }
}

impl<'a> From<&'a str> for IcalComponentName<'a> {
    fn from(name: &'a str) -> Self {
        Cow::Borrowed(name).into()
    }
}

/// The closed iCalendar component-name vocabulary, one fieldless variant per
/// known component. An identity for dispatch and nesting rules; the
/// open-vocabulary counterpart that also carries unknown names is
/// [`IcalComponentName`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalComponentKind {
    /// `VCALENDAR`: the calendar envelope (RFC 5545 3.4).
    VCalendar,
    /// `VEVENT`: an event (RFC 5545 3.6.1).
    VEvent,
    /// `VTODO`: a to-do (RFC 5545 3.6.2).
    VTodo,
    /// `VJOURNAL`: a journal entry (RFC 5545 3.6.3).
    VJournal,
    /// `VFREEBUSY`: free/busy time (RFC 5545 3.6.4).
    VFreeBusy,
    /// `VTIMEZONE`: a time-zone definition (RFC 5545 3.6.5).
    VTimezone,
    /// `STANDARD`: a standard-time rule (RFC 5545 3.6.5).
    Standard,
    /// `DAYLIGHT`: a daylight-saving-time rule (RFC 5545 3.6.5).
    Daylight,
    /// `VALARM`: an alarm (RFC 5545 3.6.6).
    VAlarm,
    /// `PARTICIPANT`: a participant (RFC 9073 7.1).
    Participant,
    /// `VLOCATION`: a location (RFC 9073 7.2).
    VLocation,
    /// `VRESOURCE`: a resource (RFC 9073 7.3).
    VResource,
}

impl IcalComponentKind {
    /// Every known component kind, for iterating the closed vocabulary.
    pub const ALL: [Self; 12] = [
        Self::VCalendar,
        Self::VEvent,
        Self::VTodo,
        Self::VJournal,
        Self::VFreeBusy,
        Self::VTimezone,
        Self::Standard,
        Self::Daylight,
        Self::VAlarm,
        Self::Participant,
        Self::VLocation,
        Self::VResource,
    ];
}

impl str::FromStr for IcalComponentKind {
    type Err = ParseIcalComponentKindError;

    /// The known component for a wire name (case-insensitive), or an error.
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        let kind = match kind {
            kind if kind.eq_ignore_ascii_case("VCALENDAR") => Self::VCalendar,
            kind if kind.eq_ignore_ascii_case("VEVENT") => Self::VEvent,
            kind if kind.eq_ignore_ascii_case("VTODO") => Self::VTodo,
            kind if kind.eq_ignore_ascii_case("VJOURNAL") => Self::VJournal,
            kind if kind.eq_ignore_ascii_case("VFREEBUSY") => Self::VFreeBusy,
            kind if kind.eq_ignore_ascii_case("VTIMEZONE") => Self::VTimezone,
            kind if kind.eq_ignore_ascii_case("STANDARD") => Self::Standard,
            kind if kind.eq_ignore_ascii_case("DAYLIGHT") => Self::Daylight,
            kind if kind.eq_ignore_ascii_case("VALARM") => Self::VAlarm,
            kind if kind.eq_ignore_ascii_case("PARTICIPANT") => Self::Participant,
            kind if kind.eq_ignore_ascii_case("VLOCATION") => Self::VLocation,
            kind if kind.eq_ignore_ascii_case("VRESOURCE") => Self::VResource,
            _ => return Err(ParseIcalComponentKindError(kind.to_string())),
        };

        Ok(kind)
    }
}

impl ops::Deref for IcalComponentKind {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::VCalendar => "VCALENDAR",
            Self::VEvent => "VEVENT",
            Self::VTodo => "VTODO",
            Self::VJournal => "VJOURNAL",
            Self::VFreeBusy => "VFREEBUSY",
            Self::VTimezone => "VTIMEZONE",
            Self::Standard => "STANDARD",
            Self::Daylight => "DAYLIGHT",
            Self::VAlarm => "VALARM",
            Self::Participant => "PARTICIPANT",
            Self::VLocation => "VLOCATION",
            Self::VResource => "VRESOURCE",
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::component::IcalComponentKind;

    #[test]
    fn round_trips_every_kind_through_its_wire_name() {
        for kind in IcalComponentKind::ALL {
            assert_eq!(IcalComponentKind::from_str(&kind).ok(), Some(kind));
        }
        assert_eq!(
            IcalComponentKind::from_str("vevent").ok(),
            Some(IcalComponentKind::VEvent),
        );
        assert!(IcalComponentKind::from_str("VUNKNOWN").is_err());
    }
}
