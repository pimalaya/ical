//! # Properties
//!
//! A decoded property and the iCalendar property-name vocabulary.
//!
//! An [`IcalProp`] is an [`IcalPropName`], a list of parameters, and a
//! decoded value. The name is stored explicitly because many properties share
//! one [`IcalValue`] kind: `SUMMARY` and `LOCATION` both decode to text, so
//! the value alone cannot say which property it is.
//!
//! A known name is held as the closed [`IcalPropKind`] identity (its wire
//! spelling reached through `Deref` and `FromStr`); an unknown one keeps its
//! verbatim bytes.
//!
//! Build a property directly from its public fields; strict, spec-checked
//! construction lives in the syntax layer
//! ([`IcalPropBuilder`](crate::tree::ical::builder::IcalPropBuilder)), which
//! this module does not depend on: pure model, no [`crate::tree`].

use core::{error, fmt, ops, str};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    param::IcalParam,
    value::{IcalValue, owned},
};

/// Parse iCalendar property kind error.
#[derive(Debug)]
pub struct ParseIcalPropKindError(
    /// The iCalendar property that cannot be parsed.
    String,
);

impl fmt::Display for ParseIcalPropKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse iCalendar property `{}`", self.0)
    }
}

impl error::Error for ParseIcalPropKindError {}

/// A decoded property: its wire name, its parameters, and its decoded value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalProp<'a> {
    /// The property name (a known kind, or an unknown name kept verbatim).
    pub name: IcalPropName<'a>,
    /// The parameters decorating the property.
    pub params: Vec<IcalParam<'a>>,
    /// The decoded value.
    pub value: IcalValue<'a>,
}

/// A property name: a known iCalendar name, or an unknown one kept verbatim.
///
/// Known names normalise to their canonical [`IcalPropKind`] spelling; unknown
/// names keep their exact bytes so they round-trip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalPropName<'a> {
    /// A name in the closed iCalendar vocabulary.
    Kind(IcalPropKind),
    /// Any other name, kept as written.
    Unknown(Cow<'a, str>),
}

impl ops::Deref for IcalPropName<'_> {
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

impl From<IcalPropKind> for IcalPropName<'_> {
    fn from(kind: IcalPropKind) -> Self {
        Self::Kind(kind)
    }
}

impl From<&IcalPropKind> for IcalPropName<'_> {
    fn from(kind: &IcalPropKind) -> Self {
        Self::Kind(*kind)
    }
}

impl<'a> From<Cow<'a, str>> for IcalPropName<'a> {
    fn from(name: Cow<'a, str>) -> Self {
        match name.parse().ok() {
            Some(kind) => Self::Kind(kind),
            None => Self::Unknown(name),
        }
    }
}

impl<'a> From<&'a str> for IcalPropName<'a> {
    fn from(name: &'a str) -> Self {
        Cow::Borrowed(name).into()
    }
}

impl IcalPropName<'_> {
    /// The same name with every borrow replaced by an allocation. See
    /// [`IcalValue::into_owned`](crate::value::IcalValue::into_owned).
    pub fn into_owned(self) -> IcalPropName<'static> {
        match self {
            Self::Kind(kind) => IcalPropName::Kind(kind),
            Self::Unknown(name) => IcalPropName::Unknown(owned(name)),
        }
    }
}

impl IcalProp<'_> {
    /// The same property with every borrow replaced by an allocation, so it
    /// outlives the bytes it was decoded from. See
    /// [`IcalValue::into_owned`](crate::value::IcalValue::into_owned).
    pub fn into_owned(self) -> IcalProp<'static> {
        IcalProp {
            name: self.name.into_owned(),
            params: self.params.into_iter().map(IcalParam::into_owned).collect(),
            value: self.value.into_owned(),
        }
    }
}

/// The closed iCalendar property-name vocabulary, one fieldless variant per
/// known property.
///
/// An identity for dispatch and allowed-sets; [`IcalPropName`] is the open
/// counterpart that also carries unknown names. Covers RFC 5545, 7986, 9073
/// and 9074, plus the vCalendar 1.0 legacy alarm properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalPropKind {
    /// `CALSCALE`: calendar scale (RFC 5545 3.7.1).
    CalScale,
    /// `METHOD`: iTIP method (RFC 5545 3.7.2).
    Method,
    /// `PRODID`: product identifier (RFC 5545 3.7.3).
    ProdId,
    /// `ATTACH`: an associated document (RFC 5545 3.8.1.1).
    Attach,
    /// `CATEGORIES`: categories or tags (RFC 5545 3.8.1.2).
    Categories,
    /// `CLASS`: access classification (RFC 5545 3.8.1.3).
    Class,
    /// `COMMENT`: a comment (RFC 5545 3.8.1.4).
    Comment,
    /// `DESCRIPTION`: a full description (RFC 5545 3.8.1.5).
    Description,
    /// `GEO`: geographic position (RFC 5545 3.8.1.6).
    Geo,
    /// `LOCATION`: the intended venue (RFC 5545 3.8.1.7).
    Location,
    /// `PERCENT-COMPLETE`: to-do completion percentage (RFC 5545 3.8.1.8).
    PercentComplete,
    /// `PRIORITY`: relative priority (RFC 5545 3.8.1.9).
    Priority,
    /// `RESOURCES`: equipment or resources (RFC 5545 3.8.1.10).
    Resources,
    /// `STATUS`: overall status (RFC 5545 3.8.1.11).
    Status,
    /// `SUMMARY`: a short summary (RFC 5545 3.8.1.12).
    Summary,
    /// `COMPLETED`: date/time a to-do was completed (RFC 5545 3.8.2.1).
    Completed,
    /// `DTEND`: end date/time (RFC 5545 3.8.2.2).
    DtEnd,
    /// `DUE`: to-do due date/time (RFC 5545 3.8.2.3).
    Due,
    /// `DTSTART`: start date/time (RFC 5545 3.8.2.4).
    DtStart,
    /// `DURATION`: a duration (RFC 5545 3.8.2.5).
    Duration,
    /// `FREEBUSY`: free/busy time (RFC 5545 3.8.2.6).
    FreeBusy,
    /// `TRANSP`: time transparency (RFC 5545 3.8.2.7).
    Transp,
    /// `TZID`: time-zone identifier (RFC 5545 3.8.3.1).
    TzId,
    /// `TZNAME`: time-zone name (RFC 5545 3.8.3.2).
    TzName,
    /// `TZOFFSETFROM`: offset in use before a transition (RFC 5545 3.8.3.3).
    TzOffsetFrom,
    /// `TZOFFSETTO`: offset in use after a transition (RFC 5545 3.8.3.4).
    TzOffsetTo,
    /// `TZURL`: time-zone definition URL (RFC 5545 3.8.3.5).
    TzUrl,
    /// `ATTENDEE`: an attendee (RFC 5545 3.8.4.1).
    Attendee,
    /// `CONTACT`: contact information (RFC 5545 3.8.4.2).
    Contact,
    /// `ORGANIZER`: the organizer (RFC 5545 3.8.4.3).
    Organizer,
    /// `RECURRENCE-ID`: identifies a recurrence instance (RFC 5545 3.8.4.4).
    RecurrenceId,
    /// `RELATED-TO`: a relationship to another component (RFC 5545 3.8.4.5).
    RelatedTo,
    /// `URL`: an associated URL (RFC 5545 3.8.4.6).
    Url,
    /// `UID`: unique identifier (RFC 5545 3.8.4.7).
    Uid,
    /// `EXDATE`: excepted recurrence dates (RFC 5545 3.8.5.1).
    ExDate,
    /// `RDATE`: recurrence dates (RFC 5545 3.8.5.2).
    RDate,
    /// `RRULE`: recurrence rule (RFC 5545 3.8.5.3).
    RRule,
    /// `EXRULE`: exception rule (RFC 2445 4.8.5.2; deprecated in RFC 5545).
    ExRule,
    /// `ACTION`: alarm action (RFC 5545 3.8.6.1).
    Action,
    /// `REPEAT`: alarm repeat count (RFC 5545 3.8.6.2).
    Repeat,
    /// `TRIGGER`: alarm trigger (RFC 5545 3.8.6.3).
    Trigger,
    /// `CREATED`: creation date/time (RFC 5545 3.8.7.1).
    Created,
    /// `DTSTAMP`: object creation/last-revision timestamp (RFC 5545 3.8.7.2).
    DtStamp,
    /// `LAST-MODIFIED`: last-modification date/time (RFC 5545 3.8.7.3).
    LastModified,
    /// `SEQUENCE`: revision sequence number (RFC 5545 3.8.7.4).
    Sequence,
    /// `REQUEST-STATUS`: scheduling request status (RFC 5545 3.8.8.3).
    RequestStatus,
    /// `NAME`: calendar display name (RFC 7986 5.1).
    Name,
    /// `REFRESH-INTERVAL`: suggested refresh interval (RFC 7986 5.7).
    RefreshInterval,
    /// `SOURCE`: calendar source URL (RFC 7986 5.8).
    Source,
    /// `COLOR`: a display colour (RFC 7986 5.9).
    Color,
    /// `IMAGE`: an associated image (RFC 7986 5.10).
    Image,
    /// `CONFERENCE`: conference access information (RFC 7986 5.11).
    Conference,
    /// `PARTICIPANT-TYPE`: participant type (RFC 9073 6.2).
    ParticipantType,
    /// `RESOURCE-TYPE`: resource type (RFC 9073 6.3).
    ResourceType,
    /// `CALENDAR-ADDRESS`: participant calendar address (RFC 9073 6.4).
    CalendarAddress,
    /// `LOCATION-TYPE`: location type (RFC 9073 6.1).
    LocationType,
    /// `STRUCTURED-DATA`: structured ancillary data (RFC 9073 6.6).
    StructuredData,
    /// `LINK`: a typed link to a related resource (RFC 9253 8.1).
    Link,
    /// `REFID`: a reference identifier grouping components (RFC 9253 8.2).
    Refid,
    /// `CONCEPT`: a categorisation of a component (RFC 9253 8.3).
    Concept,
    /// `BUSYTYPE`: the busy state an availability window states (RFC 7953 3.2).
    BusyType,
    /// `STYLED-DESCRIPTION`: rich-text description (RFC 9073 6.5).
    StyledDescription,
    /// `ACKNOWLEDGED`: alarm acknowledgement time (RFC 9074 6).
    Acknowledged,
    /// `PROXIMITY`: location-proximity trigger (RFC 9074 8).
    Proximity,
    /// `TZ`: time-zone offset (vCalendar 1.0).
    Tz,
    /// `AALARM`: audio alarm (vCalendar 1.0).
    AAlarm,
    /// `DALARM`: display alarm (vCalendar 1.0).
    DAlarm,
    /// `MALARM`: mail alarm (vCalendar 1.0).
    MAlarm,
    /// `PALARM`: procedure alarm (vCalendar 1.0).
    PAlarm,
    /// `RNUM`: recurrence-count number (vCalendar 1.0).
    RNum,
}

impl IcalPropKind {
    /// Every known property kind, for iterating the closed vocabulary (e.g. a
    /// validator checking which required properties are absent).
    pub const ALL: [Self; 70] = [
        Self::CalScale,
        Self::Method,
        Self::ProdId,
        Self::Attach,
        Self::Categories,
        Self::Class,
        Self::Comment,
        Self::Description,
        Self::Geo,
        Self::Location,
        Self::PercentComplete,
        Self::Priority,
        Self::Resources,
        Self::Status,
        Self::Summary,
        Self::Completed,
        Self::DtEnd,
        Self::Due,
        Self::DtStart,
        Self::Duration,
        Self::FreeBusy,
        Self::Transp,
        Self::TzId,
        Self::TzName,
        Self::TzOffsetFrom,
        Self::TzOffsetTo,
        Self::TzUrl,
        Self::Attendee,
        Self::Contact,
        Self::Organizer,
        Self::RecurrenceId,
        Self::RelatedTo,
        Self::Url,
        Self::Uid,
        Self::ExDate,
        Self::RDate,
        Self::RRule,
        Self::ExRule,
        Self::Action,
        Self::Repeat,
        Self::Trigger,
        Self::Created,
        Self::DtStamp,
        Self::LastModified,
        Self::Sequence,
        Self::RequestStatus,
        Self::Name,
        Self::RefreshInterval,
        Self::Source,
        Self::Color,
        Self::Image,
        Self::Conference,
        Self::ParticipantType,
        Self::ResourceType,
        Self::CalendarAddress,
        Self::LocationType,
        Self::StructuredData,
        Self::Link,
        Self::Refid,
        Self::Concept,
        Self::BusyType,
        Self::StyledDescription,
        Self::Acknowledged,
        Self::Proximity,
        Self::Tz,
        Self::AAlarm,
        Self::DAlarm,
        Self::MAlarm,
        Self::PAlarm,
        Self::RNum,
    ];
}

impl str::FromStr for IcalPropKind {
    type Err = ParseIcalPropKindError;

    /// The known property for a wire name (case-insensitive), or an error.
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        let kind = match kind {
            kind if kind.eq_ignore_ascii_case("CALSCALE") => Self::CalScale,
            kind if kind.eq_ignore_ascii_case("METHOD") => Self::Method,
            kind if kind.eq_ignore_ascii_case("PRODID") => Self::ProdId,
            kind if kind.eq_ignore_ascii_case("ATTACH") => Self::Attach,
            kind if kind.eq_ignore_ascii_case("CATEGORIES") => Self::Categories,
            kind if kind.eq_ignore_ascii_case("CLASS") => Self::Class,
            kind if kind.eq_ignore_ascii_case("COMMENT") => Self::Comment,
            kind if kind.eq_ignore_ascii_case("DESCRIPTION") => Self::Description,
            kind if kind.eq_ignore_ascii_case("GEO") => Self::Geo,
            kind if kind.eq_ignore_ascii_case("LOCATION") => Self::Location,
            kind if kind.eq_ignore_ascii_case("PERCENT-COMPLETE") => Self::PercentComplete,
            kind if kind.eq_ignore_ascii_case("PRIORITY") => Self::Priority,
            kind if kind.eq_ignore_ascii_case("RESOURCES") => Self::Resources,
            kind if kind.eq_ignore_ascii_case("STATUS") => Self::Status,
            kind if kind.eq_ignore_ascii_case("SUMMARY") => Self::Summary,
            kind if kind.eq_ignore_ascii_case("COMPLETED") => Self::Completed,
            kind if kind.eq_ignore_ascii_case("DTEND") => Self::DtEnd,
            kind if kind.eq_ignore_ascii_case("DUE") => Self::Due,
            kind if kind.eq_ignore_ascii_case("DTSTART") => Self::DtStart,
            kind if kind.eq_ignore_ascii_case("DURATION") => Self::Duration,
            kind if kind.eq_ignore_ascii_case("FREEBUSY") => Self::FreeBusy,
            kind if kind.eq_ignore_ascii_case("TRANSP") => Self::Transp,
            kind if kind.eq_ignore_ascii_case("TZID") => Self::TzId,
            kind if kind.eq_ignore_ascii_case("TZNAME") => Self::TzName,
            kind if kind.eq_ignore_ascii_case("TZOFFSETFROM") => Self::TzOffsetFrom,
            kind if kind.eq_ignore_ascii_case("TZOFFSETTO") => Self::TzOffsetTo,
            kind if kind.eq_ignore_ascii_case("TZURL") => Self::TzUrl,
            kind if kind.eq_ignore_ascii_case("ATTENDEE") => Self::Attendee,
            kind if kind.eq_ignore_ascii_case("CONTACT") => Self::Contact,
            kind if kind.eq_ignore_ascii_case("ORGANIZER") => Self::Organizer,
            kind if kind.eq_ignore_ascii_case("RECURRENCE-ID") => Self::RecurrenceId,
            kind if kind.eq_ignore_ascii_case("RELATED-TO") => Self::RelatedTo,
            kind if kind.eq_ignore_ascii_case("URL") => Self::Url,
            kind if kind.eq_ignore_ascii_case("UID") => Self::Uid,
            kind if kind.eq_ignore_ascii_case("EXDATE") => Self::ExDate,
            kind if kind.eq_ignore_ascii_case("RDATE") => Self::RDate,
            kind if kind.eq_ignore_ascii_case("RRULE") => Self::RRule,
            kind if kind.eq_ignore_ascii_case("EXRULE") => Self::ExRule,
            kind if kind.eq_ignore_ascii_case("ACTION") => Self::Action,
            kind if kind.eq_ignore_ascii_case("REPEAT") => Self::Repeat,
            kind if kind.eq_ignore_ascii_case("TRIGGER") => Self::Trigger,
            kind if kind.eq_ignore_ascii_case("CREATED") => Self::Created,
            kind if kind.eq_ignore_ascii_case("DTSTAMP") => Self::DtStamp,
            kind if kind.eq_ignore_ascii_case("LAST-MODIFIED") => Self::LastModified,
            kind if kind.eq_ignore_ascii_case("SEQUENCE") => Self::Sequence,
            kind if kind.eq_ignore_ascii_case("REQUEST-STATUS") => Self::RequestStatus,
            kind if kind.eq_ignore_ascii_case("NAME") => Self::Name,
            kind if kind.eq_ignore_ascii_case("REFRESH-INTERVAL") => Self::RefreshInterval,
            kind if kind.eq_ignore_ascii_case("SOURCE") => Self::Source,
            kind if kind.eq_ignore_ascii_case("COLOR") => Self::Color,
            kind if kind.eq_ignore_ascii_case("IMAGE") => Self::Image,
            kind if kind.eq_ignore_ascii_case("CONFERENCE") => Self::Conference,
            kind if kind.eq_ignore_ascii_case("PARTICIPANT-TYPE") => Self::ParticipantType,
            kind if kind.eq_ignore_ascii_case("RESOURCE-TYPE") => Self::ResourceType,
            kind if kind.eq_ignore_ascii_case("CALENDAR-ADDRESS") => Self::CalendarAddress,
            kind if kind.eq_ignore_ascii_case("LOCATION-TYPE") => Self::LocationType,
            kind if kind.eq_ignore_ascii_case("STRUCTURED-DATA") => Self::StructuredData,
            kind if kind.eq_ignore_ascii_case("LINK") => Self::Link,
            kind if kind.eq_ignore_ascii_case("REFID") => Self::Refid,
            kind if kind.eq_ignore_ascii_case("CONCEPT") => Self::Concept,
            kind if kind.eq_ignore_ascii_case("BUSYTYPE") => Self::BusyType,
            kind if kind.eq_ignore_ascii_case("STYLED-DESCRIPTION") => Self::StyledDescription,
            kind if kind.eq_ignore_ascii_case("ACKNOWLEDGED") => Self::Acknowledged,
            kind if kind.eq_ignore_ascii_case("PROXIMITY") => Self::Proximity,
            kind if kind.eq_ignore_ascii_case("TZ") => Self::Tz,
            kind if kind.eq_ignore_ascii_case("AALARM") => Self::AAlarm,
            kind if kind.eq_ignore_ascii_case("DALARM") => Self::DAlarm,
            kind if kind.eq_ignore_ascii_case("MALARM") => Self::MAlarm,
            kind if kind.eq_ignore_ascii_case("PALARM") => Self::PAlarm,
            kind if kind.eq_ignore_ascii_case("RNUM") => Self::RNum,
            _ => return Err(ParseIcalPropKindError(kind.to_string())),
        };

        Ok(kind)
    }
}

impl ops::Deref for IcalPropKind {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::CalScale => "CALSCALE",
            Self::Method => "METHOD",
            Self::ProdId => "PRODID",
            Self::Attach => "ATTACH",
            Self::Categories => "CATEGORIES",
            Self::Class => "CLASS",
            Self::Comment => "COMMENT",
            Self::Description => "DESCRIPTION",
            Self::Geo => "GEO",
            Self::Location => "LOCATION",
            Self::PercentComplete => "PERCENT-COMPLETE",
            Self::Priority => "PRIORITY",
            Self::Resources => "RESOURCES",
            Self::Status => "STATUS",
            Self::Summary => "SUMMARY",
            Self::Completed => "COMPLETED",
            Self::DtEnd => "DTEND",
            Self::Due => "DUE",
            Self::DtStart => "DTSTART",
            Self::Duration => "DURATION",
            Self::FreeBusy => "FREEBUSY",
            Self::Transp => "TRANSP",
            Self::TzId => "TZID",
            Self::TzName => "TZNAME",
            Self::TzOffsetFrom => "TZOFFSETFROM",
            Self::TzOffsetTo => "TZOFFSETTO",
            Self::TzUrl => "TZURL",
            Self::Attendee => "ATTENDEE",
            Self::Contact => "CONTACT",
            Self::Organizer => "ORGANIZER",
            Self::RecurrenceId => "RECURRENCE-ID",
            Self::RelatedTo => "RELATED-TO",
            Self::Url => "URL",
            Self::Uid => "UID",
            Self::ExDate => "EXDATE",
            Self::RDate => "RDATE",
            Self::RRule => "RRULE",
            Self::ExRule => "EXRULE",
            Self::Action => "ACTION",
            Self::Repeat => "REPEAT",
            Self::Trigger => "TRIGGER",
            Self::Created => "CREATED",
            Self::DtStamp => "DTSTAMP",
            Self::LastModified => "LAST-MODIFIED",
            Self::Sequence => "SEQUENCE",
            Self::RequestStatus => "REQUEST-STATUS",
            Self::Name => "NAME",
            Self::RefreshInterval => "REFRESH-INTERVAL",
            Self::Source => "SOURCE",
            Self::Color => "COLOR",
            Self::Image => "IMAGE",
            Self::Conference => "CONFERENCE",
            Self::ParticipantType => "PARTICIPANT-TYPE",
            Self::ResourceType => "RESOURCE-TYPE",
            Self::CalendarAddress => "CALENDAR-ADDRESS",
            Self::LocationType => "LOCATION-TYPE",
            Self::StructuredData => "STRUCTURED-DATA",
            Self::Link => "LINK",
            Self::Refid => "REFID",
            Self::Concept => "CONCEPT",
            Self::BusyType => "BUSYTYPE",
            Self::StyledDescription => "STYLED-DESCRIPTION",
            Self::Acknowledged => "ACKNOWLEDGED",
            Self::Proximity => "PROXIMITY",
            Self::Tz => "TZ",
            Self::AAlarm => "AALARM",
            Self::DAlarm => "DALARM",
            Self::MAlarm => "MALARM",
            Self::PAlarm => "PALARM",
            Self::RNum => "RNUM",
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use alloc::borrow::Cow;

    use crate::{
        param::IcalParam,
        prop::{IcalProp, IcalPropKind, IcalPropName},
        value::{IcalValue, text::IcalText},
    };

    #[test]
    fn names_the_property_and_wraps_the_value() {
        let prop = IcalProp {
            name: IcalPropKind::Summary.into(),
            params: [].into(),
            value: IcalValue::Text(IcalText(Cow::Borrowed("Lunch"))),
        };
        assert_eq!(prop.name, IcalPropName::Kind(IcalPropKind::Summary));
        assert_eq!(&*prop.name, "SUMMARY");
        assert!(prop.params.is_empty());
    }

    #[test]
    fn carries_the_given_parameters() {
        let prop = IcalProp {
            name: IcalPropKind::Attendee.into(),
            params: [IcalParam::Role(Cow::Borrowed("CHAIR"))].into(),
            value: IcalValue::CalAddress("mailto:a@b.example".into()),
        };
        assert_eq!(&*prop.name, "ATTENDEE");
        assert_eq!(prop.params.len(), 1);
    }

    #[test]
    fn round_trips_every_kind_through_its_wire_name() {
        for kind in IcalPropKind::ALL {
            assert_eq!(IcalPropKind::from_str(&kind).ok(), Some(kind));
        }
        // NOTE: Case-insensitive on the way in; unknown names are not in the
        // vocabulary.
        assert_eq!(
            IcalPropKind::from_str("summary").ok(),
            Some(IcalPropKind::Summary),
        );
        assert!(IcalPropKind::from_str("X-CUSTOM").is_err());
    }
}
