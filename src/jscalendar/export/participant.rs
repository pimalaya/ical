//! # Participants
//!
//! An `ORGANIZER` or `ATTENDEE` line, and a whole RFC 9073 `PARTICIPANT`
//! component, as one Participant object keyed by calendar address (RFC 8984
//! 4.4.6, bis draft 3.4.6).
//!
//! Both spellings land in one map, so an `ATTENDEE` and the `PARTICIPANT` that
//! describes the same person merge rather than appearing twice. The
//! scheduling outcome of a request travels beside them.
//!
//! JSCalendar 2.0 states the organizer as `organizerCalendarAddress` and an
//! owner Participant, which is the same one as the `ATTENDEE` of that address
//! (conversion draft 2.3.29) when the two agree on its name and email.

use alloc::{borrow::ToOwned, format, string::String, vec, vec::Vec};

use serde_json::{Map, Value, json};

use crate::{
    component::IcalComponent,
    jscalendar::{
        IcalJscalendarVersion,
        export::{
            Builder, Organizer, PARTICIPANT_PARAMS, PARTICIPANT_PARAMS_V2, component_key,
            jsid_param, key, param, set, text, values,
        },
        hatch::IcalHatch,
    },
    param::IcalParamKind,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    value::IcalValue,
};

impl Builder {
    /// `ORGANIZER` is both the reply address and a participant owning the
    /// object (draft 2.3.29).
    pub(super) fn organizer(&mut self, prop: &IcalProp<'_>) {
        let Some(address) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let key = key(&self.participants, prop);

        if self.version == IcalJscalendarVersion::V2_0 {
            let mut object = Map::new();
            object.insert("@type".to_owned(), Value::String("Participant".to_owned()));
            object.insert("calendarAddress".to_owned(), Value::String(address.clone()));
            object.insert(
                "roles".to_owned(),
                Value::Object(set(vec!["owner".to_owned()])),
            );

            for (kind, member) in [(IcalParamKind::Cn, "name"), (IcalParamKind::Email, "email")] {
                if let Some(value) = param(prop, kind) {
                    object.insert(member.to_owned(), Value::String(value.into_owned()));
                }
            }

            self.object.insert(
                "organizerCalendarAddress".to_owned(),
                Value::String(address.clone()),
            );
            self.organizer = Some(Organizer {
                key: key.clone(),
                address,
                named: jsid_param(prop).is_some(),
            });
            self.participants.insert(key, Value::Object(object));
            self.hatch.note(
                "organizerCalendarAddress",
                prop,
                &[IcalParamKind::Cn, IcalParamKind::Email],
            );

            return;
        }

        self.reply_to
            .insert("imip".to_owned(), Value::String(address.clone()));

        let mut object = participant(prop, &address, self.version);
        object.insert(
            "roles".to_owned(),
            Value::Object(set(vec!["owner".to_owned()])),
        );

        self.participants.insert(key, Value::Object(object));
        self.hatch.note("replyTo/imip", prop, PARTICIPANT_PARAMS);
    }

    /// `ATTENDEE` is a Participant (draft 2.3.4).
    pub(super) fn attendee(&mut self, prop: &IcalProp<'_>) {
        let Some(address) = text(prop) else {
            return self.hatch.keep(prop);
        };

        if self.version == IcalJscalendarVersion::V2_0 {
            return self.attendee_v2(prop, address);
        }

        let key = key(&self.participants, prop);
        let mut object = participant(prop, &address, self.version);

        let roles = set(values(prop, IcalParamKind::Role)
            .iter()
            .flat_map(|role| roles(role))
            .collect());

        object.insert(
            "roles".to_owned(),
            // NOTE: A participant must hold at least one role (RFC 8984 4.4.6).
            Value::Object(match roles.is_empty() {
                true => set(vec!["attendee".to_owned()]),
                false => roles,
            }),
        );

        if self.task
            && let Some(progress) = progress(prop)
        {
            self.object
                .insert("progress".to_owned(), Value::String(progress));
        }

        let pointer = format!("participants/{key}");
        self.participants.insert(key, Value::Object(object));
        self.hatch.note(&pointer, prop, PARTICIPANT_PARAMS);
    }

    /// A 2.0 `ATTENDEE`: no default role (bis draft A.2.3), a task's progress
    /// on the Participant (conversion draft Table 13), and the organizer's
    /// Participant joined when it is the same person.
    fn attendee_v2(&mut self, prop: &IcalProp<'_>, address: String) {
        let mut object = participant(prop, &address, self.version);

        let roles = set(values(prop, IcalParamKind::Role)
            .iter()
            .map(|role| role_v2(role))
            .collect());

        if !roles.is_empty() {
            object.insert("roles".to_owned(), Value::Object(roles));
        }

        if self.task
            && let Some(progress) = progress(prop)
        {
            object.insert(
                "participationStatus".to_owned(),
                Value::String("accepted".to_owned()),
            );
            object.insert("progress".to_owned(), Value::String(progress));
        }

        let jsid = jsid_param(prop);

        // NOTE: The organizer joins only an ATTENDEE that names it as it does,
        // so neither the name nor the key of either line is lost.
        let joined = self.organizer.as_ref().and_then(|organizer| {
            let held = self.participants.get(&organizer.key)?;
            let agrees = ["name", "email"]
                .iter()
                .all(|member| held.get(*member) == object.get(*member));
            let keyed = !organizer.named || jsid.is_none() || jsid.as_ref() == Some(&organizer.key);

            (organizer.address == address && agrees && keyed).then(|| organizer.key.clone())
        });

        let Some(held) = joined else {
            let key = key(&self.participants, prop);
            let pointer = format!("participants/{key}");
            self.participants.insert(key, Value::Object(object));
            return self.hatch.note(&pointer, prop, PARTICIPANT_PARAMS_V2);
        };

        self.organizer = None;

        if let Some(Value::Object(roles)) = object.get_mut("roles") {
            roles.insert("owner".to_owned(), Value::Bool(true));
        } else {
            object.insert(
                "roles".to_owned(),
                Value::Object(set(vec!["owner".to_owned()])),
            );
        }

        self.participants.remove(&held);
        let key = jsid.unwrap_or(held);
        let pointer = format!("participants/{key}");
        self.participants.insert(key, Value::Object(object));

        // NOTE: The record is what tells this Participant from the organizer
        // alone, which converts back to no ATTENDEE (conversion draft 3.6).
        self.hatch
            .note_always(&pointer, prop, PARTICIPANT_PARAMS_V2);
    }

    /// A `PARTICIPANT` component as a Participant (draft 2.2.1).
    pub(super) fn participant(&mut self, component: &IcalComponent<'_>) {
        let v2 = self.version == IcalJscalendarVersion::V2_0;

        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Participant".to_owned()));

        let mut hatch = IcalHatch::new("participant");
        let mut roles = Map::new();

        for prop in &component.props {
            let IcalPropName::Kind(kind) = &prop.name else {
                // NOTE: A JSID names the key its component took, and the key
                // is where that already is.
                if !prop.name.eq_ignore_ascii_case("JSID") {
                    hatch.keep(prop);
                }

                continue;
            };

            match (kind, text(prop)) {
                (IcalPropKind::CalendarAddress, Some(address)) if v2 => {
                    object.insert("calendarAddress".to_owned(), Value::String(address));
                    hatch.note("calendarAddress", prop, &[]);
                }
                (IcalPropKind::CalendarAddress, Some(address)) => {
                    object.insert(
                        "sendTo".to_owned(),
                        json!({"imip": Value::String(address.clone())}),
                    );
                    hatch.note("sendTo/imip", prop, &[]);
                }
                (IcalPropKind::Summary, Some(name)) => {
                    object.insert("name".to_owned(), Value::String(name));
                    hatch.note("name", prop, &[]);
                }
                (IcalPropKind::Description, Some(description)) => {
                    object.insert("description".to_owned(), Value::String(description));
                    hatch.note("description", prop, &[]);
                }
                (IcalPropKind::ParticipantType, Some(kind)) => {
                    roles.insert(kind.to_lowercase(), Value::Bool(true));
                    hatch.note("roles", prop, &[]);
                }
                _ => hatch.keep(prop),
            }
        }

        for child in &component.components {
            hatch.keep_component(child);
        }

        // NOTE: Roles are optional in 2.0 (bis draft 3.4.6) and `attendee` is
        // obsolete (7.6.15).
        match (roles.is_empty(), v2) {
            (true, true) => {}
            (true, false) => {
                object.insert(
                    "roles".to_owned(),
                    Value::Object(set(vec!["attendee".to_owned()])),
                );
            }
            (false, _) => {
                object.insert("roles".to_owned(), Value::Object(roles));
            }
        }

        if let Some(hatch) = hatch.into_value() {
            object.insert("iCalendar".to_owned(), hatch);
        }

        let key = component_key(&self.participants, component);
        self.participants.insert(key, Value::Object(object));
    }

    /// `REQUEST-STATUS` keeps its wire spelling (RFC 8984 4.4.7).
    pub(super) fn request_status(&mut self, prop: &IcalProp<'_>) {
        let IcalValue::RequestStatus(status) = &prop.value else {
            return self.hatch.keep(prop);
        };

        let mut text = format!("{};{}", status.code, status.description);

        if !status.extra.is_empty() {
            text.push(';');
            text.push_str(&status.extra);
        }

        self.request_status.push(Value::String(text));
        self.hatch.note("requestStatus", prop, &[]);
    }

    /// `METHOD` is the scheduling method (RFC 8984 4.1.8).
    pub(super) fn method(&mut self, prop: &IcalProp<'_>) {
        match text(prop) {
            Some(method) => self.member("method", Value::String(method.to_lowercase()), prop, &[]),
            None => self.hatch.keep(prop),
        }
    }
}

/// The Participant an `ATTENDEE` or `ORGANIZER` line describes, without its
/// roles, which the two properties word differently.
fn participant(
    prop: &IcalProp<'_>,
    address: &str,
    version: IcalJscalendarVersion,
) -> Map<String, Value> {
    let v2 = version == IcalJscalendarVersion::V2_0;

    let mut object = Map::new();
    object.insert("@type".to_owned(), Value::String("Participant".to_owned()));

    match v2 {
        true => object.insert(
            "calendarAddress".to_owned(),
            Value::String(address.to_owned()),
        ),
        false => object.insert(
            "sendTo".to_owned(),
            json!({"imip": Value::String(address.to_owned())}),
        ),
    };

    // NOTE: 2.0 obsoletes `language` and reserves the scheduling members (bis
    // draft A.2.1, A.2.2.2).
    let scalars = [
        (IcalParamKind::Cn, "name", false, true),
        (IcalParamKind::Email, "email", false, true),
        (IcalParamKind::Language, "language", false, false),
        (IcalParamKind::PartStat, "participationStatus", true, true),
        (IcalParamKind::ScheduleAgent, "scheduleAgent", true, false),
        (
            IcalParamKind::ScheduleStatus,
            "scheduleStatus",
            false,
            false,
        ),
    ];

    for (kind, member, lower, kept) in scalars {
        if v2 && !kept {
            continue;
        }

        if let Some(value) = param(prop, kind) {
            let value = match lower {
                true => value.to_lowercase(),
                false => value.into_owned(),
            };

            object.insert(member.to_owned(), Value::String(value));
        }
    }

    if let Some(cutype) = param(prop, IcalParamKind::CuType) {
        // NOTE: A room is a location to JSCalendar (draft 2.3.4).
        let kind = match cutype.to_ascii_uppercase().as_str() {
            "ROOM" => "location".to_owned(),
            _ => cutype.to_lowercase(),
        };

        object.insert("kind".to_owned(), Value::String(kind));
    }

    let flags = [
        (IcalParamKind::Rsvp, "expectReply", true),
        (IcalParamKind::ScheduleForceSend, "scheduleForceSend", false),
    ];

    for (kind, member, kept) in flags {
        if v2 && !kept {
            continue;
        }

        if let Some(flag) = param(prop, kind) {
            object.insert(
                member.to_owned(),
                Value::Bool(flag.eq_ignore_ascii_case("TRUE")),
            );
        }
    }

    let sets = [
        (IcalParamKind::DelegatedFrom, "delegatedFrom"),
        (IcalParamKind::DelegatedTo, "delegatedTo"),
        (IcalParamKind::Member, "memberOf"),
    ];

    for (kind, member) in sets {
        let addresses = set(values(prop, kind));

        if !addresses.is_empty() {
            object.insert(member.to_owned(), Value::Object(addresses));
        }
    }

    object
}

/// The JSCalendar roles one `ROLE` parameter value stands for (draft 2.3.4).
fn roles(role: &str) -> Vec<String> {
    match role.to_ascii_uppercase().as_str() {
        "REQ-PARTICIPANT" => vec!["attendee".to_owned()],
        "OPT-PARTICIPANT" => vec!["attendee".to_owned(), "optional".to_owned()],
        "NON-PARTICIPANT" => vec!["informational".to_owned()],
        "CHAIR" => vec!["attendee".to_owned(), "chair".to_owned()],
        other => vec![other.to_lowercase()],
    }
}

/// The JSCalendar 2.0 role one `ROLE` parameter value stands for (bis draft
/// 3.4.6).
fn role_v2(role: &str) -> String {
    match role.to_ascii_uppercase().as_str() {
        "REQ-PARTICIPANT" => "required".to_owned(),
        "OPT-PARTICIPANT" => "optional".to_owned(),
        "NON-PARTICIPANT" => "informational".to_owned(),
        other => other.to_lowercase(),
    }
}

/// The Task progress a `PARTSTAT` states, for the values that are specific to
/// a `VTODO` (draft 2.3.4).
fn progress(prop: &IcalProp<'_>) -> Option<String> {
    let status = param(prop, IcalParamKind::PartStat)?;

    match status.to_ascii_uppercase().as_str() {
        "COMPLETED" => Some("completed".to_owned()),
        "IN-PROCESS" => Some("in-process".to_owned()),
        "FAILED" => Some("failed".to_owned()),
        _ => None,
    }
}
