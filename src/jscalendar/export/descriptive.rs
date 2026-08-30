//! # Descriptive members
//!
//! What an object says about itself rather than about time or people: its
//! privacy, its status, its keywords, its categories and what it relates to
//! (RFC 8984 4.1, 4.3).

use alloc::{
    borrow::{Cow, ToOwned},
    format, vec,
};

use serde_json::{Map, Value};

use crate::{
    jscalendar::export::{Builder, list, param, set, text},
    param::IcalParamKind,
    prop::IcalProp,
};

impl Builder {
    /// `STYLED-DESCRIPTION` carries the description with its media type.
    pub(super) fn styled_description(&mut self, prop: &IcalProp<'_>) {
        let Some(text) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let media = param(prop, IcalParamKind::FmtType).unwrap_or(Cow::Borrowed("text/html"));

        self.object.insert(
            "descriptionContentType".to_owned(),
            Value::String(media.into_owned()),
        );
        self.member(
            "description",
            Value::String(text),
            prop,
            &[IcalParamKind::FmtType],
        );
    }

    /// `CLASS` is the privacy of the object, with `CONFIDENTIAL` spelled
    /// `secret` (RFC 8984 4.4.3).
    pub(super) fn privacy(&mut self, prop: &IcalProp<'_>) {
        let Some(class) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let privacy = match class.to_ascii_uppercase().as_str() {
            "CONFIDENTIAL" => "secret".to_owned(),
            _ => class.to_lowercase(),
        };

        self.member("privacy", Value::String(privacy), prop, &[]);
    }

    /// `STATUS` is the Event's status, and the Task's progress (draft 2.3.39).
    pub(super) fn status(&mut self, prop: &IcalProp<'_>) {
        let Some(status) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let pointer = match self.task {
            true => "progress",
            false => "status",
        };

        self.member(pointer, Value::String(status.to_lowercase()), prop, &[]);
    }

    /// `TRANSP` is the free/busy status (RFC 8984 4.4.2).
    pub(super) fn free_busy(&mut self, prop: &IcalProp<'_>) {
        let Some(transp) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let status = match transp.to_ascii_uppercase().as_str() {
            "TRANSPARENT" => "free",
            "OPAQUE" => "busy",
            _ => return self.hatch.keep(prop),
        };

        self.member(
            "freeBusyStatus",
            Value::String(status.to_owned()),
            prop,
            &[],
        );
    }

    /// `CATEGORIES` are the keywords, a set rather than a list.
    pub(super) fn keywords(&mut self, prop: &IcalProp<'_>) {
        self.keywords.extend(set(list(prop)));
        self.hatch.note("keywords", prop, &[]);
    }

    /// `CONCEPT` is a categorisation by URI (RFC 9253 8.3).
    pub(super) fn concept(&mut self, prop: &IcalProp<'_>) {
        match text(prop) {
            Some(concept) => {
                self.categories.insert(concept, Value::Bool(true));
                self.hatch.note("categories", prop, &[]);
            }
            None => self.hatch.keep(prop),
        }
    }

    /// `RELATED-TO` is a Relation keyed by the other object's `UID`.
    pub(super) fn related(&mut self, prop: &IcalProp<'_>) {
        let Some(uid) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let relation = param(prop, IcalParamKind::RelType)
            .map(|kind| set(vec![kind.to_lowercase()]))
            .unwrap_or_default();

        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Relation".to_owned()));

        if !relation.is_empty() {
            object.insert("relation".to_owned(), Value::Object(relation));
        }

        let pointer = format!("relatedTo/{uid}");
        self.related_to.insert(uid, Value::Object(object));
        self.hatch.note(&pointer, prop, &[IcalParamKind::RelType]);
    }
}
