//! # Property spec
//!
//! The per-property contract on the property markers, and the runtime vtable
//! that bridges the open [`IcalPropKind`] back to those static impls.

use crate::{
    param::{COMMON_PARAMS, IcalParamKind},
    prop::{
        IcalPropKind, aalarm, acknowledged, action, attach, attendee, busytype, calendar_address,
        calscale, cardinality::IcalPropCardinality, categories, class, color, comment, completed,
        concept, conference, contact, created, dalarm, description, dtend, dtstamp, dtstart, due,
        duration, exdate, exrule, freebusy, geo, image, last_modified, link, location,
        location_type, malarm, method, name, organizer, palarm, participant_type, percent_complete,
        priority, prodid, proximity, rdate, recurrence_id, refid, refresh_interval, related_to,
        repeat, request_status, resource_type, resources, rnum, rrule, sequence, source, status,
        structured_data, styled_description, summary, transp, trigger, tz, tzid, tzname,
        tzoffsetfrom, tzoffsetto, tzuntil, tzurl, uid, url,
    },
    value::IcalValueKind,
    version::IcalVersion,
};

/// The per-property contract: the versions it lives in, its multiplicity, the
/// value types and parameters it may carry, all per version.
///
/// Implemented on the zero-sized property markers. The defaults cover the
/// uniform majority (a single text value, valid in every version), so a
/// property overrides only where it diverges; the only required item is
/// [`KIND`](Self::KIND).
pub trait IcalPropSpec {
    /// The property this spec describes.
    const KIND: IcalPropKind;

    /// The versions in which the property is defined (the existence axis).
    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0, IcalVersion::V2_0]
    }

    /// How many times the property may appear in its component, in the given
    /// version. Most are repeatable; the single-valued ones override this.
    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::Any
    }

    /// The value-types the property may take, default-first, for the version.
    /// Index 0 is the type used when no `VALUE` is declared.
    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Text]
    }

    /// The parameters the property may carry, in the given version.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        COMMON_PARAMS
    }

    /// The value-type in force: the declared `VALUE` kind if any, else the
    /// version default, else [`Text`](IcalValueKind::Text). Liberal: a declared
    /// kind outside `allowed_values` is honoured here.
    fn value(version: IcalVersion, declared: Option<IcalValueKind>) -> IcalValueKind {
        declared
            .or_else(|| Self::allowed_values(version).first().copied())
            .unwrap_or(IcalValueKind::Text)
    }
}

/// The spec of a property as function pointers, the runtime bridge from the
/// open [`IcalPropKind`] back to the static per-marker [`IcalPropSpec`] impls.
#[allow(dead_code)]
pub(crate) struct IcalPropSpecFns {
    /// The property this spec describes, so the dispatch can be checked against
    /// itself.
    pub kind: IcalPropKind,
    /// See [`IcalPropSpec::allowed_versions`].
    pub allowed_versions: fn() -> &'static [IcalVersion],
    /// See [`IcalPropSpec::cardinality`].
    pub cardinality: fn(IcalVersion) -> IcalPropCardinality,
    /// See [`IcalPropSpec::allowed_values`].
    pub allowed_values: fn(IcalVersion) -> &'static [IcalValueKind],
    /// See [`IcalPropSpec::allowed_params`].
    pub allowed_params: fn(IcalVersion) -> &'static [IcalParamKind],
    /// See [`IcalPropSpec::value`].
    pub value: fn(IcalVersion, Option<IcalValueKind>) -> IcalValueKind,
}

/// Collect the spec function pointers of a marker type.
const fn spec_fns<L: IcalPropSpec>() -> IcalPropSpecFns {
    IcalPropSpecFns {
        kind: L::KIND,
        allowed_versions: L::allowed_versions,
        cardinality: L::cardinality,
        allowed_values: L::allowed_values,
        allowed_params: L::allowed_params,
        value: L::value,
    }
}

/// Dispatch a property kind onto its marker spec.
///
/// Each spec is a constant, so the dispatch hands out a reference: a spec
/// returned by value goes through memory and the call through its pointer
/// waits on the stores.
pub(crate) fn prop_spec(prop: IcalPropKind) -> &'static IcalPropSpecFns {
    use IcalPropKind::*;

    match prop {
        CalScale => &const { spec_fns::<calscale::CALSCALE>() },
        Method => &const { spec_fns::<method::METHOD>() },
        ProdId => &const { spec_fns::<prodid::PRODID>() },
        Attach => &const { spec_fns::<attach::ATTACH>() },
        Categories => &const { spec_fns::<categories::CATEGORIES>() },
        Class => &const { spec_fns::<class::CLASS>() },
        Comment => &const { spec_fns::<comment::COMMENT>() },
        Description => &const { spec_fns::<description::DESCRIPTION>() },
        Geo => &const { spec_fns::<geo::GEO>() },
        Location => &const { spec_fns::<location::LOCATION>() },
        PercentComplete => &const { spec_fns::<percent_complete::PERCENT_COMPLETE>() },
        Priority => &const { spec_fns::<priority::PRIORITY>() },
        Resources => &const { spec_fns::<resources::RESOURCES>() },
        Status => &const { spec_fns::<status::STATUS>() },
        Summary => &const { spec_fns::<summary::SUMMARY>() },
        Completed => &const { spec_fns::<completed::COMPLETED>() },
        DtEnd => &const { spec_fns::<dtend::DTEND>() },
        Due => &const { spec_fns::<due::DUE>() },
        DtStart => &const { spec_fns::<dtstart::DTSTART>() },
        Duration => &const { spec_fns::<duration::DURATION>() },
        FreeBusy => &const { spec_fns::<freebusy::FREEBUSY>() },
        Transp => &const { spec_fns::<transp::TRANSP>() },
        TzId => &const { spec_fns::<tzid::TZID>() },
        TzName => &const { spec_fns::<tzname::TZNAME>() },
        TzOffsetFrom => &const { spec_fns::<tzoffsetfrom::TZOFFSETFROM>() },
        TzOffsetTo => &const { spec_fns::<tzoffsetto::TZOFFSETTO>() },
        TzUrl => &const { spec_fns::<tzurl::TZURL>() },
        TzUntil => &const { spec_fns::<tzuntil::TZUNTIL>() },
        Attendee => &const { spec_fns::<attendee::ATTENDEE>() },
        Contact => &const { spec_fns::<contact::CONTACT>() },
        Organizer => &const { spec_fns::<organizer::ORGANIZER>() },
        RecurrenceId => &const { spec_fns::<recurrence_id::RECURRENCE_ID>() },
        RelatedTo => &const { spec_fns::<related_to::RELATED_TO>() },
        Url => &const { spec_fns::<url::URL>() },
        Uid => &const { spec_fns::<uid::UID>() },
        ExDate => &const { spec_fns::<exdate::EXDATE>() },
        RDate => &const { spec_fns::<rdate::RDATE>() },
        RRule => &const { spec_fns::<rrule::RRULE>() },
        ExRule => &const { spec_fns::<exrule::EXRULE>() },
        Action => &const { spec_fns::<action::ACTION>() },
        Repeat => &const { spec_fns::<repeat::REPEAT>() },
        Trigger => &const { spec_fns::<trigger::TRIGGER>() },
        Created => &const { spec_fns::<created::CREATED>() },
        DtStamp => &const { spec_fns::<dtstamp::DTSTAMP>() },
        LastModified => &const { spec_fns::<last_modified::LAST_MODIFIED>() },
        Sequence => &const { spec_fns::<sequence::SEQUENCE>() },
        RequestStatus => &const { spec_fns::<request_status::REQUEST_STATUS>() },
        Name => &const { spec_fns::<name::NAME>() },
        RefreshInterval => &const { spec_fns::<refresh_interval::REFRESH_INTERVAL>() },
        Source => &const { spec_fns::<source::SOURCE>() },
        Color => &const { spec_fns::<color::COLOR>() },
        Image => &const { spec_fns::<image::IMAGE>() },
        Conference => &const { spec_fns::<conference::CONFERENCE>() },
        ParticipantType => &const { spec_fns::<participant_type::PARTICIPANT_TYPE>() },
        ResourceType => &const { spec_fns::<resource_type::RESOURCE_TYPE>() },
        CalendarAddress => &const { spec_fns::<calendar_address::CALENDAR_ADDRESS>() },
        LocationType => &const { spec_fns::<location_type::LOCATION_TYPE>() },
        StructuredData => &const { spec_fns::<structured_data::STRUCTURED_DATA>() },
        Link => &const { spec_fns::<link::LINK>() },
        Refid => &const { spec_fns::<refid::REFID>() },
        Concept => &const { spec_fns::<concept::CONCEPT>() },
        BusyType => &const { spec_fns::<busytype::BUSYTYPE>() },
        StyledDescription => &const { spec_fns::<styled_description::STYLED_DESCRIPTION>() },
        Acknowledged => &const { spec_fns::<acknowledged::ACKNOWLEDGED>() },
        Proximity => &const { spec_fns::<proximity::PROXIMITY>() },
        Tz => &const { spec_fns::<tz::TZ>() },
        AAlarm => &const { spec_fns::<aalarm::AALARM>() },
        DAlarm => &const { spec_fns::<dalarm::DALARM>() },
        MAlarm => &const { spec_fns::<malarm::MALARM>() },
        PAlarm => &const { spec_fns::<palarm::PALARM>() },
        RNum => &const { spec_fns::<rnum::RNUM>() },
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prop::{IcalPropKind, spec::prop_spec},
        value::IcalValueKind,
        version::IcalVersion,
    };

    #[test]
    fn dispatches_every_property_onto_its_own_marker() {
        for kind in IcalPropKind::ALL {
            assert_eq!(prop_spec(kind).kind, kind, "{}", &*kind);
        }
    }

    #[test]
    fn every_property_states_a_value_kind_it_allows() {
        for kind in IcalPropKind::ALL {
            let spec = prop_spec(kind);

            for version in (spec.allowed_versions)() {
                let allowed = (spec.allowed_values)(*version);
                let in_force = (spec.value)(*version, None);

                // NOTE: With nothing declared, the kind in force is the first
                // allowed one, so an empty allowed set would make the decoder
                // fall back to text behind the spec's back.
                assert!(!allowed.is_empty(), "{} allows no value kind", &*kind);
                assert!(
                    allowed.contains(&in_force),
                    "{} decodes as {} which it does not allow",
                    &*kind,
                    &*in_force,
                );
            }
        }
    }

    #[test]
    fn a_declared_kind_wins_over_the_default() {
        let spec = prop_spec(IcalPropKind::Attach);

        assert_eq!(
            (spec.value)(IcalVersion::V2_0, None),
            IcalValueKind::Uri,
            "the default is the first allowed kind"
        );
        assert_eq!(
            (spec.value)(IcalVersion::V2_0, Some(IcalValueKind::Integer)),
            IcalValueKind::Integer,
            "a declared kind is honoured even outside the allowed set"
        );
    }

    #[test]
    fn a_list_property_stays_a_list_whatever_is_declared() {
        let spec = prop_spec(IcalPropKind::RDate);

        // NOTE: The declared kind describes each item, not the value as a
        // whole, so RDATE;VALUE=PERIOD is still a list of periods.
        assert_eq!(
            (spec.value)(IcalVersion::V2_0, Some(IcalValueKind::Period)),
            IcalValueKind::DateTimeList
        );
    }
}
