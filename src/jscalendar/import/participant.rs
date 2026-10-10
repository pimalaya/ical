//! # Participants
//!
//! A Participant object read back as an `ATTENDEE` or `ORGANIZER` line, and
//! as the RFC 9073 `PARTICIPANT` component where it says more than a line
//! can carry.

use alloc::{
    borrow::{Cow, ToOwned},
    format,
    string::String,
    vec::Vec,
};

use serde_json::{Map, Value};

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    jscalendar::{
        hatch::{hatch_of, kept_components, kept_props},
        import::{keyed, keys, named, plain, text_prop},
    },
    param::IcalParam,
    prop::{IcalProp, IcalPropKind},
    value::{IcalValue, cal_address::IcalCalAddress},
    version::IcalVersion,
};

/// A Participant as an `ATTENDEE` or `ORGANIZER` property, or as the
/// `PARTICIPANT` component it came from when it owns a hatch of its own.
///
/// `organizing` is set for a JSCalendar 2.0 object, saying whether this is
/// the organizer's Participant: there an `ATTENDEE` is always an `ATTENDEE`,
/// the `ORGANIZER` coming from `organizerCalendarAddress` instead.
pub(super) fn participant(
    hatch: Option<&Map<String, Value>>,
    component: &str,
    key: &str,
    participant: &Value,
    organizing: Option<bool>,
) -> Result<IcalProp<'static>, IcalComponent<'static>> {
    let address = address(participant).unwrap_or_default();

    if participant.get("iCalendar").is_some() {
        return Err(vparticipant(participant, address));
    }

    let owner = participant
        .get("roles")
        .and_then(Value::as_object)
        .is_some_and(|roles| roles.contains_key("owner"));

    // NOTE: An owning participant is the ORGANIZER, and that is where the
    // export recorded its leftovers: under the reply address it also wrote, not
    // under the participant.
    let (pointer, fallback) = match owner && organizing.is_none() {
        true => ("replyTo/imip".to_owned(), IcalPropKind::Organizer),
        false => (format!("participants/{key}"), IcalPropKind::Attendee),
    };

    let mut prop = text_prop(
        named(hatch, component, &pointer, fallback),
        address.to_owned(),
    );
    prop.value = IcalValue::CalAddress(IcalCalAddress(Cow::Owned(address.to_owned())));

    let scalars = [
        ("name", 0usize),
        ("email", 1),
        ("language", 2),
        ("participationStatus", 3),
        ("kind", 4),
        ("scheduleAgent", 5),
        ("scheduleStatus", 6),
    ];

    for (member, slot) in scalars {
        let Some(text) = participant.get(member).and_then(Value::as_str) else {
            continue;
        };

        let upper = Cow::Owned(text.to_ascii_uppercase());

        prop.params.push(match slot {
            0 => IcalParam::Cn(Cow::Owned(text.to_owned())),
            1 => IcalParam::Email(Cow::Owned(text.to_owned())),
            2 => IcalParam::Language(Cow::Owned(text.to_owned())),
            3 => IcalParam::PartStat(upper),
            4 => IcalParam::CuType(match text {
                "location" => Cow::Borrowed("ROOM"),
                _ => upper,
            }),
            5 => IcalParam::ScheduleAgent(upper),
            _ => IcalParam::ScheduleStatus(Cow::Owned(text.to_owned())),
        });
    }

    let flags = [("expectReply", false), ("scheduleForceSend", true)];

    for (member, forced) in flags {
        let Some(flag) = participant.get(member).and_then(Value::as_bool) else {
            continue;
        };

        let text = match flag {
            true => Cow::Borrowed("TRUE"),
            false => Cow::Borrowed("FALSE"),
        };

        prop.params.push(match forced {
            true => IcalParam::ScheduleForceSend(text),
            false => IcalParam::Rsvp(text),
        });
    }

    let roles: Vec<String> = keys(participant.get("roles").unwrap_or(&Value::Null)).collect();

    let role = match organizing {
        Some(organizing) => role_v2(&roles, organizing),
        None => role(&roles),
    };

    if let Some(role) = role {
        prop.params.push(IcalParam::Role(Cow::Owned(role)));
    }

    // NOTE: A 2.0 task attendee's progress is a PARTSTAT of its own, beside an
    // `accepted` participation status (conversion draft Table 13).
    if organizing.is_some()
        && let Some(progress) = participant.get("progress").and_then(Value::as_str)
    {
        prop.params
            .retain(|param| !matches!(param, IcalParam::PartStat(_)));
        prop.params.push(IcalParam::PartStat(Cow::Owned(
            progress.to_ascii_uppercase(),
        )));
    }

    let sets = [
        ("delegatedFrom", 0usize),
        ("delegatedTo", 1),
        ("memberOf", 2),
    ];

    for (member, slot) in sets {
        let addresses: Vec<Cow<'static, str>> =
            keys(participant.get(member).unwrap_or(&Value::Null))
                .map(Cow::Owned)
                .collect();

        if addresses.is_empty() {
            continue;
        }

        prop.params.push(match slot {
            0 => IcalParam::DelegatedFrom(addresses),
            1 => IcalParam::DelegatedTo(addresses),
            _ => IcalParam::Member(addresses),
        });
    }

    Ok(keyed(prop, key))
}

/// A Participant that carries its own escape hatch, as a `PARTICIPANT`
/// component.
pub(super) fn vparticipant(participant: &Value, address: &str) -> IcalComponent<'static> {
    let hatch = participant.as_object().and_then(hatch_of);

    let mut props = Vec::new();

    if !address.is_empty() {
        props.push(plain(IcalPropKind::CalendarAddress, address.to_owned()));
    }

    if let Some(name) = participant.get("name").and_then(Value::as_str) {
        props.push(plain(IcalPropKind::Summary, name.to_owned()));
    }

    if let Some(description) = participant.get("description").and_then(Value::as_str) {
        props.push(plain(IcalPropKind::Description, description.to_owned()));
    }

    for role in keys(participant.get("roles").unwrap_or(&Value::Null)) {
        props.push(plain(
            IcalPropKind::ParticipantType,
            role.to_ascii_uppercase(),
        ));
    }

    props.extend(
        kept_props(hatch, IcalVersion::V2_0)
            .into_iter()
            .map(IcalProp::into_owned),
    );

    IcalComponent {
        name: IcalComponentName::Kind(IcalComponentKind::Participant),
        props,
        components: kept_components(hatch, IcalVersion::V2_0)
            .into_iter()
            .map(IcalComponent::into_owned)
            .collect(),
    }
}

/// The `ROLE` one set of JSCalendar roles states, if iCalendar has a word for
/// it (draft 2.3.4).
pub(super) fn role(roles: &[String]) -> Option<String> {
    let has = |role: &str| roles.iter().any(|held| held == role);

    let role = match (has("chair"), has("optional"), has("informational")) {
        (true, _, _) => "CHAIR",
        (_, true, _) => "OPT-PARTICIPANT",
        (_, _, true) => "NON-PARTICIPANT",
        // NOTE: `attendee` alone is the default role, which iCalendar leaves
        // unwritten; `owner` is carried by ORGANIZER rather than by ROLE.
        _ => return None,
    };

    Some(role.to_owned())
}

/// The JSCalendar 2.0 `ROLE`: the first of `chair`, `required`, `optional`
/// and `informational` the set holds (bis draft 3.4.6), else a role iCalendar
/// has no word for, else `OWNER` for an owner that is not the organizer
/// (conversion draft 2.3.4).
pub(super) fn role_v2(roles: &[String], organizing: bool) -> Option<String> {
    let has = |role: &str| roles.iter().any(|held| held == role);

    let known = [
        ("chair", "CHAIR"),
        ("required", "REQ-PARTICIPANT"),
        ("optional", "OPT-PARTICIPANT"),
        ("informational", "NON-PARTICIPANT"),
    ];

    if let Some((_, role)) = known.iter().find(|(role, _)| has(role)) {
        return Some((*role).to_owned());
    }

    // NOTE: `attendee` is RFC 8984's default role, which iCalendar leaves
    // unwritten.
    let other = roles
        .iter()
        .find(|role| !matches!(role.as_str(), "attendee" | "owner"));

    match (other, has("owner") && !organizing) {
        (Some(role), _) => Some(role.to_ascii_uppercase()),
        (None, true) => Some("OWNER".to_owned()),
        (None, false) => None,
    }
}

/// The calendar address a Participant states: its 2.0 `calendarAddress`, else
/// its RFC 8984 `sendTo` iMIP one.
pub(super) fn address(participant: &Value) -> Option<&str> {
    participant
        .get("calendarAddress")
        .or_else(|| participant.get("sendTo").and_then(|to| to.get("imip")))
        .and_then(Value::as_str)
}

/// Whether the organizer's Participant says nothing an `ORGANIZER` cannot, so
/// it converts to no `ATTENDEE` (conversion draft 3.6).
///
/// The export records a Participant that was an `ATTENDEE` as well, which is
/// how a plain attendee organizing its own event keeps its line.
pub(super) fn represented(
    hatch: Option<&Map<String, Value>>,
    key: &str,
    participant: &Value,
) -> bool {
    const MEMBERS: [&str; 6] = [
        "@type",
        "calendarAddress",
        "sendTo",
        "roles",
        "name",
        "email",
    ];

    let Some(object) = participant.as_object() else {
        return false;
    };

    let owner = object
        .get("roles")
        .and_then(Value::as_object)
        .is_some_and(|roles| roles.len() == 1 && roles.contains_key("owner"));

    let recorded = hatch
        .and_then(|hatch| hatch.get("convertedProperties"))
        .and_then(Value::as_object)
        .is_some_and(|converted| converted.contains_key(&format!("participants/{key}")));

    owner
        && !recorded
        && object
            .keys()
            .all(|member| MEMBERS.contains(&member.as_str()))
}

/// The 2.0 `ORGANIZER`, from `organizerCalendarAddress`, with the name and
/// email of the organizer's Participant where the record carries none.
pub(super) fn organizer(
    hatch: Option<&Map<String, Value>>,
    component: &str,
    address: &str,
    participant: Option<&Value>,
) -> IcalProp<'static> {
    let mut prop = text_prop(
        named(
            hatch,
            component,
            "organizerCalendarAddress",
            IcalPropKind::Organizer,
        ),
        address.to_owned(),
    );
    prop.value = IcalValue::CalAddress(IcalCalAddress(Cow::Owned(address.to_owned())));

    for (member, slot) in [("name", 0usize), ("email", 1)] {
        let Some(text) = participant
            .and_then(|participant| participant.get(member))
            .and_then(Value::as_str)
        else {
            continue;
        };

        let param = match slot {
            0 => IcalParam::Cn(Cow::Owned(text.to_owned())),
            _ => IcalParam::Email(Cow::Owned(text.to_owned())),
        };

        if !prop.params.iter().any(|held| held.kind() == param.kind()) {
            prop.params.push(param);
        }
    }

    prop
}
