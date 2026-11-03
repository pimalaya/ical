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
//! construction is [`IcalPropBuilder`](crate::builder::IcalPropBuilder).
//!
//! This module also owns the per-property contract: one marker module per
//! property the crate knows, each carrying its [`IcalPropSpec`](spec) impl,
//! and the [`spec`] vtable that bridges the open [`IcalPropKind`] back to
//! those static impls. The contract is what the RFC allows, so it is model
//! rather than syntax and needs no parser; the read-and-edit lens on the same
//! marker lives in [`crate::tree::prop`].

pub mod aalarm;
pub mod acknowledged;
pub mod action;
pub mod attach;
pub mod attendee;
pub mod busytype;
pub mod calendar_address;
pub mod calscale;
pub mod cardinality;
pub mod categories;
pub mod class;
pub mod color;
pub mod comment;
pub mod completed;
pub mod concept;
pub mod conference;
pub mod contact;
pub mod created;
pub mod dalarm;
pub mod description;
pub mod dtend;
pub mod dtstamp;
pub mod dtstart;
pub mod due;
pub mod duration;
pub mod exdate;
pub mod exrule;
pub mod freebusy;
pub mod geo;
pub mod image;
pub mod last_modified;
pub mod link;
pub mod location;
pub mod location_type;
pub mod malarm;
pub mod method;
pub mod name;
pub mod organizer;
pub mod palarm;
pub mod participant_type;
pub mod percent_complete;
pub mod priority;
pub mod prodid;
pub mod proximity;
pub mod rdate;
pub mod recurrence_id;
pub mod refid;
pub mod refresh_interval;
pub mod related_to;
pub mod repeat;
pub mod request_status;
pub mod resource_type;
pub mod resources;
pub mod rnum;
pub mod rrule;
pub mod sequence;
pub mod source;
pub mod spec;
pub mod status;
pub mod structured_data;
pub mod styled_description;
pub mod summary;
pub mod transp;
pub mod trigger;
pub mod tz;
pub mod tzid;
pub mod tzname;
pub mod tzoffsetfrom;
pub mod tzoffsetto;
pub mod tzuntil;
pub mod tzurl;
pub mod uid;
pub mod url;

use core::{error, fmt, ops, str};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    param::IcalParam,
    value::{IcalUnknownValue, IcalValue, owned, text::IcalText},
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
        match IcalPropKind::from_name(&name) {
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

impl<'a> IcalProp<'a> {
    /// A text property: a known kind or an `X-` name, its parameters and its
    /// raw value, escaped when the calendar is serialized.
    ///
    /// The shape a projection mints when it synthesizes a calendar from
    /// another representation.
    pub fn text(
        name: impl Into<IcalPropName<'a>>,
        params: Vec<IcalParam<'a>>,
        value: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            name: name.into(),
            params,
            value: IcalValue::Text(IcalText(value.into())),
        }
    }
}

impl IcalProp<'_> {
    /// A property holding nothing, for the decoder to fill in place.
    pub(crate) const fn empty() -> Self {
        Self {
            name: IcalPropName::Unknown(Cow::Borrowed("")),
            params: Vec::new(),
            value: IcalValue::Unknown(IcalUnknownValue {
                components: Vec::new(),
            }),
        }
    }

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
/// and 9074, RFC 7808's `TZUNTIL`, plus the vCalendar 1.0 legacy alarm
/// properties.
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
    /// `TZUNTIL`: the exclusive end of a time-zone definition (RFC 7808 7.1).
    TzUntil,
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
    pub const ALL: [Self; 71] = [
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
        Self::TzUntil,
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

impl IcalPropKind {
    /// The known property for a wire name (case-insensitive), or `None`.
    ///
    /// The name is matched as it is first, the canonical upper case being the
    /// usual spelling, and only a name holding a lower-case letter is
    /// upper-cased into a stack copy and matched again: the copy is read back
    /// before its stores land. No allocation, for an unknown name either.
    pub(crate) fn from_name(kind: &str) -> Option<Self> {
        if let Some(kind) = Self::from_upper(kind.as_bytes()) {
            return Some(kind);
        }

        if !kind.bytes().any(|byte| byte.is_ascii_lowercase()) {
            return None;
        }

        let mut upper = [0; 18];
        let upper = upper.get_mut(..kind.len())?;
        upper.copy_from_slice(kind.as_bytes());
        upper.make_ascii_uppercase();

        Self::from_upper(upper)
    }

    /// The known property for an upper-case wire name, or `None`.
    fn from_upper(kind: &[u8]) -> Option<Self> {
        Some(match kind {
            b"CALSCALE" => Self::CalScale,
            b"METHOD" => Self::Method,
            b"PRODID" => Self::ProdId,
            b"ATTACH" => Self::Attach,
            b"CATEGORIES" => Self::Categories,
            b"CLASS" => Self::Class,
            b"COMMENT" => Self::Comment,
            b"DESCRIPTION" => Self::Description,
            b"GEO" => Self::Geo,
            b"LOCATION" => Self::Location,
            b"PERCENT-COMPLETE" => Self::PercentComplete,
            b"PRIORITY" => Self::Priority,
            b"RESOURCES" => Self::Resources,
            b"STATUS" => Self::Status,
            b"SUMMARY" => Self::Summary,
            b"COMPLETED" => Self::Completed,
            b"DTEND" => Self::DtEnd,
            b"DUE" => Self::Due,
            b"DTSTART" => Self::DtStart,
            b"DURATION" => Self::Duration,
            b"FREEBUSY" => Self::FreeBusy,
            b"TRANSP" => Self::Transp,
            b"TZID" => Self::TzId,
            b"TZNAME" => Self::TzName,
            b"TZOFFSETFROM" => Self::TzOffsetFrom,
            b"TZOFFSETTO" => Self::TzOffsetTo,
            b"TZURL" => Self::TzUrl,
            b"TZUNTIL" => Self::TzUntil,
            b"ATTENDEE" => Self::Attendee,
            b"CONTACT" => Self::Contact,
            b"ORGANIZER" => Self::Organizer,
            b"RECURRENCE-ID" => Self::RecurrenceId,
            b"RELATED-TO" => Self::RelatedTo,
            b"URL" => Self::Url,
            b"UID" => Self::Uid,
            b"EXDATE" => Self::ExDate,
            b"RDATE" => Self::RDate,
            b"RRULE" => Self::RRule,
            b"EXRULE" => Self::ExRule,
            b"ACTION" => Self::Action,
            b"REPEAT" => Self::Repeat,
            b"TRIGGER" => Self::Trigger,
            b"CREATED" => Self::Created,
            b"DTSTAMP" => Self::DtStamp,
            b"LAST-MODIFIED" => Self::LastModified,
            b"SEQUENCE" => Self::Sequence,
            b"REQUEST-STATUS" => Self::RequestStatus,
            b"NAME" => Self::Name,
            b"REFRESH-INTERVAL" => Self::RefreshInterval,
            b"SOURCE" => Self::Source,
            b"COLOR" => Self::Color,
            b"IMAGE" => Self::Image,
            b"CONFERENCE" => Self::Conference,
            b"PARTICIPANT-TYPE" => Self::ParticipantType,
            b"RESOURCE-TYPE" => Self::ResourceType,
            b"CALENDAR-ADDRESS" => Self::CalendarAddress,
            b"LOCATION-TYPE" => Self::LocationType,
            b"STRUCTURED-DATA" => Self::StructuredData,
            b"LINK" => Self::Link,
            b"REFID" => Self::Refid,
            b"CONCEPT" => Self::Concept,
            b"BUSYTYPE" => Self::BusyType,
            b"STYLED-DESCRIPTION" => Self::StyledDescription,
            b"ACKNOWLEDGED" => Self::Acknowledged,
            b"PROXIMITY" => Self::Proximity,
            b"TZ" => Self::Tz,
            b"AALARM" => Self::AAlarm,
            b"DALARM" => Self::DAlarm,
            b"MALARM" => Self::MAlarm,
            b"PALARM" => Self::PAlarm,
            b"RNUM" => Self::RNum,
            _ => return None,
        })
    }
}

impl str::FromStr for IcalPropKind {
    type Err = ParseIcalPropKindError;

    /// The known property for a wire name (case-insensitive), or an error.
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        Self::from_name(kind).ok_or_else(|| ParseIcalPropKindError(kind.to_string()))
    }
}

impl ops::Deref for IcalPropKind {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.wire_name()
    }
}

impl IcalPropKind {
    /// The name as written on the wire, in its canonical spelling.
    pub(crate) const fn wire_name(self) -> &'static str {
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
            Self::TzUntil => "TZUNTIL",
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
        assert_eq!(
            IcalPropKind::from_str("summary").ok(),
            Some(IcalPropKind::Summary),
        );
        assert!(IcalPropKind::from_str("X-CUSTOM").is_err());
    }
}
