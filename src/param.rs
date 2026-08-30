//! # Parameters
//!
//! A decoded parameter and the iCalendar parameter-name vocabulary.
//!
//! [`IcalParam`] is a closed set of the parameters RFC 5545 and its
//! extensions define, one variant each, plus an
//! [`Unknown`](IcalParam::Unknown) arm so anything else round-trips.
//!
//! Parameters are few and simple (a text or a small list), so unlike
//! properties each variant carries its value directly rather than through a
//! shared value type; the variant itself names the parameter.
//!
//! A known name is the closed [`IcalParamKind`], reached through `FromStr`
//! and `Deref`. Pure model, no [`crate::tree`] dependency.
//!
//! The default parameter set the property spec starts from lives here too
//! (`COMMON_PARAMS`), the contract being model rather than syntax; the
//! read-and-write lens on each parameter is in [`crate::tree::param`].

use core::{error, fmt, ops, str};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::value::owned;

/// Parse iCalendar parameter kind error.
#[derive(Debug)]
pub struct ParseIcalParamKindError(
    /// The iCalendar parameter that cannot be parsed.
    String,
);

impl fmt::Display for ParseIcalParamKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse iCalendar parameter `{}`", self.0)
    }
}

impl error::Error for ParseIcalParamKindError {}

/// The closed iCalendar parameter-name vocabulary, one fieldless variant per
/// known parameter. An identity for dispatch and allowed-sets; the open
/// counterpart that carries the value (and unknown names) is [`IcalParam`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalParamKind {
    /// `ALTREP`: alternate text representation URI (RFC 5545 3.2.1).
    AltRep,
    /// `CN`: common name of a calendar user (RFC 5545 3.2.2).
    Cn,
    /// `CUTYPE`: calendar user type (RFC 5545 3.2.3).
    CuType,
    /// `DELEGATED-FROM`: delegators of an attendee (RFC 5545 3.2.4).
    DelegatedFrom,
    /// `DELEGATED-TO`: delegatees of an attendee (RFC 5545 3.2.5).
    DelegatedTo,
    /// `DIR`: directory-entry reference URI (RFC 5545 3.2.6).
    Dir,
    /// `ENCODING`: inline encoding of the value (RFC 5545 3.2.7).
    Encoding,
    /// `FMTTYPE`: media type of a referenced object (RFC 5545 3.2.8).
    FmtType,
    /// `FBTYPE`: free/busy time type (RFC 5545 3.2.9).
    FbType,
    /// `LANGUAGE`: language of the value (RFC 5545 3.2.10).
    Language,
    /// `MEMBER`: group memberships of an attendee (RFC 5545 3.2.11).
    Member,
    /// `PARTSTAT`: participation status (RFC 5545 3.2.12).
    PartStat,
    /// `RANGE`: recurrence-instance range (RFC 5545 3.2.13).
    Range,
    /// `RELATED`: alarm trigger relationship (RFC 5545 3.2.14).
    Related,
    /// `RELTYPE`: relationship type (RFC 5545 3.2.15).
    RelType,
    /// `ROLE`: participation role (RFC 5545 3.2.16).
    Role,
    /// `RSVP`: RSVP expectation (RFC 5545 3.2.17).
    Rsvp,
    /// `SENT-BY`: calendar user acting on behalf of another (RFC 5545 3.2.18).
    SentBy,
    /// `TZID`: reference to a time-zone definition (RFC 5545 3.2.19).
    TzId,
    /// `VALUE`: value type the value is to be read as (RFC 5545 3.2.20).
    Value,
    /// `DISPLAY`: image display type (RFC 7986 6.1).
    Display,
    /// `EMAIL`: email address of a calendar user (RFC 7986 6.2).
    Email,
    /// `FEATURE`: conference feature set (RFC 7986 6.3).
    Feature,
    /// `LABEL`: human-readable label (RFC 7986 6.4).
    Label,
    /// `ORDER`: ordering among like properties (RFC 9073 5.1).
    Order,
    /// `SCHEMA`: identifies structured-data content (RFC 9073 5.2).
    Schema,
    /// `DERIVED`: marks a derived property value (RFC 9073 5.3).
    Derived,
    /// `SCHEDULE-AGENT`: who performs scheduling for an attendee (RFC 6638
    /// 7.1).
    ScheduleAgent,
    /// `SCHEDULE-FORCE-SEND`: a request to resend a scheduling message (RFC
    /// 6638 7.2).
    ScheduleForceSend,
    /// `SCHEDULE-STATUS`: the status of a scheduling operation (RFC 6638 7.3).
    ScheduleStatus,
    /// `LINKREL`: the relation type of a `LINK` (RFC 9253 6.1).
    LinkRel,
    /// `GAP`: the lag or lead between two related components (RFC 9253 6.2).
    Gap,
    /// `CHARSET`: character set of the value (vCalendar 1.0).
    Charset,
}

impl IcalParamKind {
    /// Every known parameter kind, for iterating the closed vocabulary, as
    /// [`IcalPropKind::ALL`](crate::prop::IcalPropKind::ALL) does for
    /// properties.
    pub const ALL: [Self; 33] = [
        Self::AltRep,
        Self::Cn,
        Self::CuType,
        Self::DelegatedFrom,
        Self::DelegatedTo,
        Self::Dir,
        Self::Encoding,
        Self::FmtType,
        Self::FbType,
        Self::Language,
        Self::Member,
        Self::PartStat,
        Self::Range,
        Self::Related,
        Self::RelType,
        Self::Role,
        Self::Rsvp,
        Self::SentBy,
        Self::TzId,
        Self::Value,
        Self::Display,
        Self::Email,
        Self::Feature,
        Self::Label,
        Self::Order,
        Self::Schema,
        Self::Derived,
        Self::ScheduleAgent,
        Self::ScheduleForceSend,
        Self::ScheduleStatus,
        Self::LinkRel,
        Self::Gap,
        Self::Charset,
    ];
}

impl str::FromStr for IcalParamKind {
    type Err = ParseIcalParamKindError;

    /// The known parameter for a wire name (case-insensitive).
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        let kind = match kind {
            kind if kind.eq_ignore_ascii_case("ALTREP") => Self::AltRep,
            kind if kind.eq_ignore_ascii_case("CN") => Self::Cn,
            kind if kind.eq_ignore_ascii_case("CUTYPE") => Self::CuType,
            kind if kind.eq_ignore_ascii_case("DELEGATED-FROM") => Self::DelegatedFrom,
            kind if kind.eq_ignore_ascii_case("DELEGATED-TO") => Self::DelegatedTo,
            kind if kind.eq_ignore_ascii_case("DIR") => Self::Dir,
            kind if kind.eq_ignore_ascii_case("ENCODING") => Self::Encoding,
            kind if kind.eq_ignore_ascii_case("FMTTYPE") => Self::FmtType,
            kind if kind.eq_ignore_ascii_case("FBTYPE") => Self::FbType,
            kind if kind.eq_ignore_ascii_case("LANGUAGE") => Self::Language,
            kind if kind.eq_ignore_ascii_case("MEMBER") => Self::Member,
            kind if kind.eq_ignore_ascii_case("PARTSTAT") => Self::PartStat,
            kind if kind.eq_ignore_ascii_case("RANGE") => Self::Range,
            kind if kind.eq_ignore_ascii_case("RELATED") => Self::Related,
            kind if kind.eq_ignore_ascii_case("RELTYPE") => Self::RelType,
            kind if kind.eq_ignore_ascii_case("ROLE") => Self::Role,
            kind if kind.eq_ignore_ascii_case("RSVP") => Self::Rsvp,
            kind if kind.eq_ignore_ascii_case("SENT-BY") => Self::SentBy,
            kind if kind.eq_ignore_ascii_case("TZID") => Self::TzId,
            kind if kind.eq_ignore_ascii_case("VALUE") => Self::Value,
            kind if kind.eq_ignore_ascii_case("DISPLAY") => Self::Display,
            kind if kind.eq_ignore_ascii_case("EMAIL") => Self::Email,
            kind if kind.eq_ignore_ascii_case("FEATURE") => Self::Feature,
            kind if kind.eq_ignore_ascii_case("LABEL") => Self::Label,
            kind if kind.eq_ignore_ascii_case("ORDER") => Self::Order,
            kind if kind.eq_ignore_ascii_case("SCHEMA") => Self::Schema,
            kind if kind.eq_ignore_ascii_case("DERIVED") => Self::Derived,
            kind if kind.eq_ignore_ascii_case("SCHEDULE-AGENT") => Self::ScheduleAgent,
            kind if kind.eq_ignore_ascii_case("SCHEDULE-FORCE-SEND") => Self::ScheduleForceSend,
            kind if kind.eq_ignore_ascii_case("SCHEDULE-STATUS") => Self::ScheduleStatus,
            kind if kind.eq_ignore_ascii_case("LINKREL") => Self::LinkRel,
            kind if kind.eq_ignore_ascii_case("GAP") => Self::Gap,
            kind if kind.eq_ignore_ascii_case("CHARSET") => Self::Charset,
            _ => return Err(ParseIcalParamKindError(kind.to_string())),
        };

        Ok(kind)
    }
}

impl ops::Deref for IcalParamKind {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::AltRep => "ALTREP",
            Self::Cn => "CN",
            Self::CuType => "CUTYPE",
            Self::DelegatedFrom => "DELEGATED-FROM",
            Self::DelegatedTo => "DELEGATED-TO",
            Self::Dir => "DIR",
            Self::Encoding => "ENCODING",
            Self::FmtType => "FMTTYPE",
            Self::FbType => "FBTYPE",
            Self::Language => "LANGUAGE",
            Self::Member => "MEMBER",
            Self::PartStat => "PARTSTAT",
            Self::Range => "RANGE",
            Self::Related => "RELATED",
            Self::RelType => "RELTYPE",
            Self::Role => "ROLE",
            Self::Rsvp => "RSVP",
            Self::SentBy => "SENT-BY",
            Self::TzId => "TZID",
            Self::Value => "VALUE",
            Self::Display => "DISPLAY",
            Self::Email => "EMAIL",
            Self::Feature => "FEATURE",
            Self::Label => "LABEL",
            Self::Order => "ORDER",
            Self::Schema => "SCHEMA",
            Self::Derived => "DERIVED",
            Self::ScheduleAgent => "SCHEDULE-AGENT",
            Self::ScheduleForceSend => "SCHEDULE-FORCE-SEND",
            Self::ScheduleStatus => "SCHEDULE-STATUS",
            Self::LinkRel => "LINKREL",
            Self::Gap => "GAP",
            Self::Charset => "CHARSET",
        }
    }
}

/// A decoded parameter: one known kind, or `Unknown` for anything unmodelled.
/// The list-valued parameters (`DELEGATED-FROM`, `DELEGATED-TO`, `MEMBER`,
/// `FEATURE`) carry a vector; the rest carry a single value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalParam<'a> {
    /// `ALTREP`: an alternate text representation URI.
    AltRep(Cow<'a, str>),
    /// `CN`: the common name of a calendar user.
    Cn(Cow<'a, str>),
    /// `CUTYPE`: the calendar user type (e.g. `INDIVIDUAL`, `ROOM`).
    CuType(Cow<'a, str>),
    /// `DELEGATED-FROM`: the calendar users this attendee is delegated from.
    DelegatedFrom(Vec<Cow<'a, str>>),
    /// `DELEGATED-TO`: the calendar users this attendee is delegated to.
    DelegatedTo(Vec<Cow<'a, str>>),
    /// `DIR`: a directory-entry reference URI.
    Dir(Cow<'a, str>),
    /// `ENCODING`: the inline encoding of the value (`8BIT`, `BASE64`).
    Encoding(Cow<'a, str>),
    /// `FMTTYPE`: the media type of a referenced object.
    FmtType(Cow<'a, str>),
    /// `FBTYPE`: the free/busy time type (`FREE`, `BUSY`, ...).
    FbType(Cow<'a, str>),
    /// `LANGUAGE`: the language of the value (RFC 5646 tag).
    Language(Cow<'a, str>),
    /// `MEMBER`: the groups this attendee is a member of.
    Member(Vec<Cow<'a, str>>),
    /// `PARTSTAT`: the participation status (`ACCEPTED`, `DECLINED`, ...).
    PartStat(Cow<'a, str>),
    /// `RANGE`: the recurrence-instance range (`THISANDFUTURE`).
    Range(Cow<'a, str>),
    /// `RELATED`: the alarm trigger relationship (`START`, `END`).
    Related(Cow<'a, str>),
    /// `RELTYPE`: the relationship type (`PARENT`, `CHILD`, `SIBLING`).
    RelType(Cow<'a, str>),
    /// `ROLE`: the participation role (`CHAIR`, `REQ-PARTICIPANT`, ...).
    Role(Cow<'a, str>),
    /// `RSVP`: whether an RSVP is expected (`TRUE`, `FALSE`).
    Rsvp(Cow<'a, str>),
    /// `SENT-BY`: the calendar user acting on behalf of another.
    SentBy(Cow<'a, str>),
    /// `TZID`: the referenced time-zone identifier.
    TzId(Cow<'a, str>),
    /// `VALUE`: the value type the property value is to be read as.
    Value(Cow<'a, str>),
    /// `DISPLAY`: the image display type (`BADGE`, `GRAPHIC`, ...).
    Display(Cow<'a, str>),
    /// `EMAIL`: the email address of a calendar user.
    Email(Cow<'a, str>),
    /// `FEATURE`: the conference feature set (`AUDIO`, `VIDEO`, ...).
    Feature(Vec<Cow<'a, str>>),
    /// `LABEL`: a human-readable label.
    Label(Cow<'a, str>),
    /// `ORDER`: the ordering among like properties (a positive integer).
    Order(Cow<'a, str>),
    /// `SCHEMA`: the URI identifying structured-data content.
    Schema(Cow<'a, str>),
    /// `DERIVED`: whether the property value is derived (`TRUE`, `FALSE`).
    Derived(Cow<'a, str>),
    /// `SCHEDULE-AGENT`: who performs scheduling for an attendee.
    ScheduleAgent(Cow<'a, str>),
    /// `SCHEDULE-FORCE-SEND`: a request to resend a scheduling message.
    ScheduleForceSend(Cow<'a, str>),
    /// `SCHEDULE-STATUS`: the status of a scheduling operation.
    ScheduleStatus(Cow<'a, str>),
    /// `LINKREL`: the relation type of a `LINK`.
    LinkRel(Cow<'a, str>),
    /// `GAP`: the lag or lead between two related components.
    Gap(Cow<'a, str>),
    /// `CHARSET`: the character set of the value (vCalendar 1.0).
    Charset(Cow<'a, str>),
    /// Any parameter the model does not decode: its name and its values.
    Unknown {
        /// The verbatim parameter name.
        name: Cow<'a, str>,
        /// The parameter values, in source order.
        values: Vec<Cow<'a, str>>,
    },
}

impl IcalParam<'_> {
    /// The closed [`IcalParamKind`] of this parameter, or `None` for
    /// [`Unknown`](IcalParam::Unknown) (which is outside the vocabulary).
    pub fn kind(&self) -> Option<IcalParamKind> {
        match self {
            Self::AltRep(_) => Some(IcalParamKind::AltRep),
            Self::Cn(_) => Some(IcalParamKind::Cn),
            Self::CuType(_) => Some(IcalParamKind::CuType),
            Self::DelegatedFrom(_) => Some(IcalParamKind::DelegatedFrom),
            Self::DelegatedTo(_) => Some(IcalParamKind::DelegatedTo),
            Self::Dir(_) => Some(IcalParamKind::Dir),
            Self::Encoding(_) => Some(IcalParamKind::Encoding),
            Self::FmtType(_) => Some(IcalParamKind::FmtType),
            Self::FbType(_) => Some(IcalParamKind::FbType),
            Self::Language(_) => Some(IcalParamKind::Language),
            Self::Member(_) => Some(IcalParamKind::Member),
            Self::PartStat(_) => Some(IcalParamKind::PartStat),
            Self::Range(_) => Some(IcalParamKind::Range),
            Self::Related(_) => Some(IcalParamKind::Related),
            Self::RelType(_) => Some(IcalParamKind::RelType),
            Self::Role(_) => Some(IcalParamKind::Role),
            Self::Rsvp(_) => Some(IcalParamKind::Rsvp),
            Self::SentBy(_) => Some(IcalParamKind::SentBy),
            Self::TzId(_) => Some(IcalParamKind::TzId),
            Self::Value(_) => Some(IcalParamKind::Value),
            Self::Display(_) => Some(IcalParamKind::Display),
            Self::Email(_) => Some(IcalParamKind::Email),
            Self::Feature(_) => Some(IcalParamKind::Feature),
            Self::Label(_) => Some(IcalParamKind::Label),
            Self::Order(_) => Some(IcalParamKind::Order),
            Self::Schema(_) => Some(IcalParamKind::Schema),
            Self::Derived(_) => Some(IcalParamKind::Derived),
            Self::ScheduleAgent(_) => Some(IcalParamKind::ScheduleAgent),
            Self::ScheduleForceSend(_) => Some(IcalParamKind::ScheduleForceSend),
            Self::ScheduleStatus(_) => Some(IcalParamKind::ScheduleStatus),
            Self::LinkRel(_) => Some(IcalParamKind::LinkRel),
            Self::Gap(_) => Some(IcalParamKind::Gap),
            Self::Charset(_) => Some(IcalParamKind::Charset),
            Self::Unknown { .. } => None,
        }
    }

    /// The same parameter with every borrow replaced by an allocation, so it
    /// outlives the bytes it was decoded from. See
    /// [`IcalValue::into_owned`](crate::value::IcalValue::into_owned).
    pub fn into_owned(self) -> IcalParam<'static> {
        use IcalParam::*;

        match self {
            AltRep(value) => AltRep(owned(value)),
            Cn(value) => Cn(owned(value)),
            CuType(value) => CuType(owned(value)),
            DelegatedFrom(values) => DelegatedFrom(values.into_iter().map(owned).collect()),
            DelegatedTo(values) => DelegatedTo(values.into_iter().map(owned).collect()),
            Dir(value) => Dir(owned(value)),
            Encoding(value) => Encoding(owned(value)),
            FmtType(value) => FmtType(owned(value)),
            FbType(value) => FbType(owned(value)),
            Language(value) => Language(owned(value)),
            Member(values) => Member(values.into_iter().map(owned).collect()),
            PartStat(value) => PartStat(owned(value)),
            Range(value) => Range(owned(value)),
            Related(value) => Related(owned(value)),
            RelType(value) => RelType(owned(value)),
            Role(value) => Role(owned(value)),
            Rsvp(value) => Rsvp(owned(value)),
            SentBy(value) => SentBy(owned(value)),
            TzId(value) => TzId(owned(value)),
            Value(value) => Value(owned(value)),
            Display(value) => Display(owned(value)),
            Email(value) => Email(owned(value)),
            Feature(values) => Feature(values.into_iter().map(owned).collect()),
            Label(value) => Label(owned(value)),
            Order(value) => Order(owned(value)),
            Schema(value) => Schema(owned(value)),
            Derived(value) => Derived(owned(value)),
            ScheduleAgent(value) => ScheduleAgent(owned(value)),
            ScheduleForceSend(value) => ScheduleForceSend(owned(value)),
            ScheduleStatus(value) => ScheduleStatus(owned(value)),
            LinkRel(value) => LinkRel(owned(value)),
            Gap(value) => Gap(owned(value)),
            Charset(value) => Charset(owned(value)),
            Unknown { name, values } => Unknown {
                name: owned(name),
                values: values.into_iter().map(owned).collect(),
            },
        }
    }
}

/// The default parameters a property may carry, used by the spec for the
/// uniform majority. Per-property sets refine this where a property allows
/// more or fewer.
pub(crate) const COMMON_PARAMS: &[IcalParamKind] = &[
    IcalParamKind::Value,
    IcalParamKind::Language,
    IcalParamKind::AltRep,
];

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::param::IcalParamKind;

    #[test]
    fn round_trips_every_kind_through_its_wire_name() {
        for kind in [
            IcalParamKind::Role,
            IcalParamKind::DelegatedFrom,
            IcalParamKind::TzId,
        ] {
            assert_eq!(IcalParamKind::from_str(&kind).ok(), Some(kind));
        }
        assert_eq!(
            IcalParamKind::from_str("role").ok(),
            Some(IcalParamKind::Role),
        );
        assert!(IcalParamKind::from_str("X-CUSTOM").is_err());
    }
}
