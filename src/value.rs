//! # Property values
//!
//! The decoded value of a property, one variant per iCalendar value kind.
//!
//! [`IcalValue`] is the semantic counterpart of a content line's raw value (the
//! syntactic [`IcalValueNode`](crate::tree::value::IcalValueNode)). Most
//! properties share a small set of value kinds: a single text, a text list, a
//! URI, a date/time, an integer. A handful are genuinely structured and get
//! their own bespoke types ([`geo::IcalGeo`],
//! [`request_status::IcalRequestStatus`]). Anything the model does not decode
//! falls back to [`Unknown`](IcalValue::Unknown), which keeps the raw
//! components so it round-trips.
//!
//! These types carry no wire name and no escaping: the property name lives on
//! [`IcalProp::name`](crate::prop::IcalProp::name), and the escaping and
//! framing live on the syntax side ([`crate::tree`]). That keeps the whole
//! decoded model free of any dependency on `tree`, so it can be used on its
//! own.

pub mod binary;
pub mod boolean;
pub mod cal_address;
pub mod datetime;
pub mod duration;
pub mod float;
pub mod geo;
pub mod integer;
pub mod period;
pub mod recur;
pub mod request_status;
pub mod text;
pub mod uri;
pub mod utc_offset;

use core::{error, fmt, ops, str};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::value::{
    binary::IcalBinary,
    boolean::IcalBoolean,
    cal_address::IcalCalAddress,
    datetime::{IcalDate, IcalDateTime, IcalDateTimeList, IcalTime},
    duration::IcalDuration,
    float::IcalFloat,
    geo::IcalGeo,
    integer::IcalInteger,
    period::IcalPeriod,
    recur::IcalRecur,
    request_status::IcalRequestStatus,
    text::{IcalText, IcalTextList},
    uri::IcalUri,
    utc_offset::IcalUtcOffset,
};

/// Parse iCalendar value kind error.
#[derive(Debug)]
pub struct ParseIcalValueKindError(
    /// The iCalendar value type that cannot be parsed.
    String,
);

impl fmt::Display for ParseIcalValueKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse iCalendar value type `{}`", self.0)
    }
}

impl error::Error for ParseIcalValueKindError {}

/// The closed iCalendar value-type vocabulary (RFC 5545 3.3 and extensions),
/// one fieldless variant per value kind. It is the discriminant of
/// [`IcalValue`] (which also has an `Unknown` arm outside this closed set) and
/// the currency of the prop spec's allowed-values sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalValueKind {
    /// An inline-base64 or URI-reference binary value (RFC 5545 3.3.1).
    Binary,
    /// A boolean value (RFC 5545 3.3.2).
    Boolean,
    /// A calendar user address, a URI (RFC 5545 3.3.3).
    CalAddress,
    /// A calendar date (RFC 5545 3.3.4).
    Date,
    /// A date with time (RFC 5545 3.3.5).
    DateTime,
    /// A comma-separated list of dates, date-times or periods, as `RDATE` and
    /// `EXDATE` carry (RFC 5545 3.8.5.1, 3.8.5.2).
    DateTimeList,
    /// A duration (RFC 5545 3.3.6).
    Duration,
    /// A floating-point number (RFC 5545 3.3.7).
    Float,
    /// The structured `GEO` latitude/longitude pair (RFC 5545 3.8.1.6).
    Geo,
    /// A signed integer (RFC 5545 3.3.8).
    Integer,
    /// A period of time (RFC 5545 3.3.9).
    Period,
    /// A recurrence rule (RFC 5545 3.3.10).
    Recur,
    /// The structured `REQUEST-STATUS` value (RFC 5545 3.8.8.3).
    RequestStatus,
    /// A single text value (RFC 5545 3.3.11).
    Text,
    /// A comma-separated text list (RFC 5545 3.3.11).
    TextList,
    /// A time of day (RFC 5545 3.3.12).
    Time,
    /// A URI (RFC 5545 3.3.13).
    Uri,
    /// A UTC offset (RFC 5545 3.3.14).
    UtcOffset,
}

impl IcalValueKind {
    /// Every known value kind, for iterating the closed vocabulary, as
    /// [`IcalPropKind::ALL`](crate::prop::IcalPropKind::ALL) does for
    /// properties.
    pub const ALL: [Self; 18] = [
        Self::Binary,
        Self::Boolean,
        Self::CalAddress,
        Self::Date,
        Self::DateTime,
        Self::DateTimeList,
        Self::Duration,
        Self::Float,
        Self::Geo,
        Self::Integer,
        Self::Period,
        Self::Recur,
        Self::RequestStatus,
        Self::Text,
        Self::TextList,
        Self::Time,
        Self::Uri,
        Self::UtcOffset,
    ];
}

impl str::FromStr for IcalValueKind {
    type Err = ParseIcalValueKindError;

    /// The value kind named by a `VALUE` parameter (case-insensitive). Liberal:
    /// it maps every wire spelling onto a model kind, leaving membership checks
    /// to a later validation tier.
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        match kind {
            kind if kind.eq_ignore_ascii_case("BINARY") => Ok(Self::Binary),
            kind if kind.eq_ignore_ascii_case("BOOLEAN") => Ok(Self::Boolean),
            kind if kind.eq_ignore_ascii_case("CAL-ADDRESS") => Ok(Self::CalAddress),
            kind if kind.eq_ignore_ascii_case("DATE") => Ok(Self::Date),
            kind if kind.eq_ignore_ascii_case("DATE-TIME") => Ok(Self::DateTime),
            kind if kind.eq_ignore_ascii_case("DATE-TIME-LIST") => Ok(Self::DateTimeList),
            kind if kind.eq_ignore_ascii_case("DURATION") => Ok(Self::Duration),
            kind if kind.eq_ignore_ascii_case("FLOAT") => Ok(Self::Float),
            kind if kind.eq_ignore_ascii_case("GEO") => Ok(Self::Geo),
            kind if kind.eq_ignore_ascii_case("INTEGER") => Ok(Self::Integer),
            kind if kind.eq_ignore_ascii_case("PERIOD") => Ok(Self::Period),
            kind if kind.eq_ignore_ascii_case("RECUR") => Ok(Self::Recur),
            kind if kind.eq_ignore_ascii_case("REQUEST-STATUS") => Ok(Self::RequestStatus),
            kind if kind.eq_ignore_ascii_case("TEXT") => Ok(Self::Text),
            kind if kind.eq_ignore_ascii_case("TEXT-LIST") => Ok(Self::TextList),
            kind if kind.eq_ignore_ascii_case("TIME") => Ok(Self::Time),
            kind if kind.eq_ignore_ascii_case("URI") => Ok(Self::Uri),
            kind if kind.eq_ignore_ascii_case("UTC-OFFSET") => Ok(Self::UtcOffset),
            _ => Err(ParseIcalValueKindError(kind.to_string())),
        }
    }
}

impl ops::Deref for IcalValueKind {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Binary => "BINARY",
            Self::Boolean => "BOOLEAN",
            Self::CalAddress => "CAL-ADDRESS",
            Self::Date => "DATE",
            Self::DateTime => "DATE-TIME",
            Self::DateTimeList => "DATE-TIME-LIST",
            Self::Duration => "DURATION",
            Self::Float => "FLOAT",
            Self::Geo => "GEO",
            Self::Integer => "INTEGER",
            Self::Period => "PERIOD",
            Self::Recur => "RECUR",
            Self::RequestStatus => "REQUEST-STATUS",
            Self::Text => "TEXT",
            Self::TextList => "TEXT-LIST",
            Self::Time => "TIME",
            Self::Uri => "URI",
            Self::UtcOffset => "UTC-OFFSET",
        }
    }
}

/// A decoded property value: one known kind, or `Unknown` (raw) for anything
/// the model does not decode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalValue<'a> {
    /// A binary value (`ATTACH`, `IMAGE`): a URI reference or inline base64.
    Binary(IcalBinary<'a>),
    /// A boolean value.
    Boolean(IcalBoolean<'a>),
    /// A calendar user address (`ORGANIZER`, `ATTENDEE`).
    CalAddress(IcalCalAddress<'a>),
    /// A calendar date.
    Date(IcalDate<'a>),
    /// A date with time (`DTSTAMP`, `DTSTART`, ...).
    DateTime(IcalDateTime<'a>),
    /// A list of dates, date-times or periods (`RDATE`, `EXDATE`).
    DateTimeList(IcalDateTimeList<'a>),
    /// A duration (`DURATION`, `TRIGGER`).
    Duration(IcalDuration<'a>),
    /// A floating-point number.
    Float(IcalFloat<'a>),
    /// The structured `GEO` latitude/longitude pair.
    Geo(IcalGeo<'a>),
    /// A signed integer (`PRIORITY`, `SEQUENCE`, `PERCENT-COMPLETE`).
    Integer(IcalInteger<'a>),
    /// A period of time (`FREEBUSY`).
    Period(IcalPeriod<'a>),
    /// A recurrence rule (`RRULE`).
    Recur(IcalRecur<'a>),
    /// The structured `REQUEST-STATUS` value.
    RequestStatus(IcalRequestStatus<'a>),
    /// A single text value (`SUMMARY`, `DESCRIPTION`, ...).
    Text(IcalText<'a>),
    /// A comma-separated text list (`CATEGORIES`, `RESOURCES`).
    TextList(IcalTextList<'a>),
    /// A time of day.
    Time(IcalTime<'a>),
    /// A URI (`URL`, `TZURL`, `SOURCE`, ...).
    Uri(IcalUri<'a>),
    /// A UTC offset (`TZOFFSETFROM`, `TZOFFSETTO`).
    UtcOffset(IcalUtcOffset<'a>),

    /// Any value the model does not decode, kept as its raw components so it
    /// round-trips.
    Unknown(IcalUnknownValue<'a>),
}

impl IcalValue<'_> {
    /// The closed [`IcalValueKind`] of this value, or `None` for
    /// [`Unknown`](IcalValue::Unknown) (which is outside the vocabulary).
    pub fn kind(&self) -> Option<IcalValueKind> {
        match self {
            Self::Binary(_) => Some(IcalValueKind::Binary),
            Self::Boolean(_) => Some(IcalValueKind::Boolean),
            Self::CalAddress(_) => Some(IcalValueKind::CalAddress),
            Self::Date(_) => Some(IcalValueKind::Date),
            Self::DateTime(_) => Some(IcalValueKind::DateTime),
            Self::DateTimeList(_) => Some(IcalValueKind::DateTimeList),
            Self::Duration(_) => Some(IcalValueKind::Duration),
            Self::Float(_) => Some(IcalValueKind::Float),
            Self::Geo(_) => Some(IcalValueKind::Geo),
            Self::Integer(_) => Some(IcalValueKind::Integer),
            Self::Period(_) => Some(IcalValueKind::Period),
            Self::Recur(_) => Some(IcalValueKind::Recur),
            Self::RequestStatus(_) => Some(IcalValueKind::RequestStatus),
            Self::Text(_) => Some(IcalValueKind::Text),
            Self::TextList(_) => Some(IcalValueKind::TextList),
            Self::Time(_) => Some(IcalValueKind::Time),
            Self::Uri(_) => Some(IcalValueKind::Uri),
            Self::UtcOffset(_) => Some(IcalValueKind::UtcOffset),
            Self::Unknown(_) => None,
        }
    }

    /// The same value with every borrow replaced by an allocation, so it
    /// outlives the bytes it was decoded from.
    ///
    /// The counterpart of [`Cow::into_owned`](alloc::borrow::Cow::into_owned),
    /// for a whole value: a calendar read from a buffer that is about to go
    /// away, or rebuilt from data that was never one line to begin with, needs
    /// exactly this.
    pub fn into_owned(self) -> IcalValue<'static> {
        match self {
            Self::Binary(IcalBinary::Uri(value)) => {
                IcalValue::Binary(IcalBinary::Uri(owned(value)))
            }
            Self::Binary(IcalBinary::Base64(value)) => {
                IcalValue::Binary(IcalBinary::Base64(owned(value)))
            }
            Self::Boolean(value) => IcalValue::Boolean(IcalBoolean(owned(value.0))),
            Self::CalAddress(value) => IcalValue::CalAddress(IcalCalAddress(owned(value.0))),
            Self::Date(value) => IcalValue::Date(IcalDate(owned(value.0))),
            Self::DateTime(value) => IcalValue::DateTime(IcalDateTime(owned(value.0))),
            Self::DateTimeList(value) => {
                IcalValue::DateTimeList(IcalDateTimeList(value.0.into_iter().map(owned).collect()))
            }
            Self::Duration(value) => IcalValue::Duration(IcalDuration(owned(value.0))),
            Self::Float(value) => IcalValue::Float(IcalFloat(owned(value.0))),
            Self::Geo(value) => IcalValue::Geo(IcalGeo {
                latitude: owned(value.latitude),
                longitude: owned(value.longitude),
            }),
            Self::Integer(value) => IcalValue::Integer(IcalInteger(owned(value.0))),
            Self::Period(value) => IcalValue::Period(IcalPeriod(owned(value.0))),
            Self::Recur(value) => IcalValue::Recur(IcalRecur(owned(value.0))),
            Self::RequestStatus(value) => IcalValue::RequestStatus(IcalRequestStatus {
                code: owned(value.code),
                description: owned(value.description),
                extra: owned(value.extra),
            }),
            Self::Text(value) => IcalValue::Text(IcalText(owned(value.0))),
            Self::TextList(value) => {
                IcalValue::TextList(IcalTextList(value.0.into_iter().map(owned).collect()))
            }
            Self::Time(value) => IcalValue::Time(IcalTime(owned(value.0))),
            Self::Uri(value) => IcalValue::Uri(IcalUri(owned(value.0))),
            Self::UtcOffset(value) => IcalValue::UtcOffset(IcalUtcOffset(owned(value.0))),
            Self::Unknown(value) => IcalValue::Unknown(IcalUnknownValue {
                components: value
                    .components
                    .into_iter()
                    .map(|component| component.into_iter().map(owned).collect())
                    .collect(),
            }),
        }
    }
}

/// One borrowed string as an owned one, the step every `into_owned` in the
/// model is made of.
pub(crate) fn owned(text: Cow<'_, str>) -> Cow<'static, str> {
    Cow::Owned(text.into_owned())
}

/// An undecoded property value: its unescaped components, in source order. The
/// property name lives on [`IcalProp::name`](crate::prop::IcalProp::name).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalUnknownValue<'a> {
    /// The value, as components of values.
    pub components: Vec<Vec<Cow<'a, str>>>,
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::value::{IcalUnknownValue, IcalValue, IcalValueKind, text::IcalText};

    #[test]
    fn reports_the_kind_of_a_value_and_none_for_unknown() {
        assert_eq!(
            IcalValue::Text(IcalText::default()).kind(),
            Some(IcalValueKind::Text),
        );
        assert_eq!(IcalValue::Unknown(IcalUnknownValue::default()).kind(), None);
    }

    #[test]
    fn maps_value_param_strings_liberally_and_case_insensitively() {
        assert_eq!("URI".parse().ok(), Some(IcalValueKind::Uri));
        assert_eq!("date-time".parse().ok(), Some(IcalValueKind::DateTime));
        assert!(IcalValueKind::from_str("bogus").is_err());
    }
}
