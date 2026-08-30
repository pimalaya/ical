//! # Places and links
//!
//! Where an object happens and what it points at: a `LOCATION` or `VLOCATION`
//! as a Location object (RFC 8984 4.2.5), a `CONFERENCE` as a virtual location
//! (4.2.6), and an `ATTACH`, `IMAGE`, `URL` or `LINK` as a Link (4.2.7).

use alloc::{borrow::ToOwned, format};

use serde_json::{Map, Value};

use crate::{
    component::IcalComponent,
    jscalendar::{
        export::{Builder, component_key, key, list, param, set, text, values},
        hatch::IcalHatch,
    },
    param::IcalParamKind,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    value::IcalValue,
};

impl Builder {
    /// `LOCATION` is a named Location (RFC 8984 4.2.5).
    pub(super) fn location(&mut self, prop: &IcalProp<'_>) {
        let Some(name) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let key = key(&self.locations, prop);
        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Location".to_owned()));
        object.insert("name".to_owned(), Value::String(name));

        let pointer = format!("locations/{key}");
        self.locations.insert(key, Value::Object(object));
        self.hatch.note(&pointer, prop, &[]);
    }

    /// `GEO` is a Location holding only coordinates, as a `geo:` URI.
    pub(super) fn geo(&mut self, prop: &IcalProp<'_>) {
        let IcalValue::Geo(geo) = &prop.value else {
            return self.hatch.keep(prop);
        };

        let key = key(&self.locations, prop);
        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Location".to_owned()));
        object.insert(
            "coordinates".to_owned(),
            Value::String(format!("geo:{},{}", geo.latitude, geo.longitude)),
        );

        let pointer = format!("locations/{key}/coordinates");
        self.locations.insert(key, Value::Object(object));
        self.hatch.note(&pointer, prop, &[]);
    }

    /// `CONFERENCE` is a VirtualLocation (RFC 8984 4.2.6).
    pub(super) fn conference(&mut self, prop: &IcalProp<'_>) {
        let Some(uri) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let key = key(&self.virtual_locations, prop);
        let mut object = Map::new();
        object.insert(
            "@type".to_owned(),
            Value::String("VirtualLocation".to_owned()),
        );
        object.insert("uri".to_owned(), Value::String(uri));

        if let Some(label) = param(prop, IcalParamKind::Label) {
            object.insert("name".to_owned(), Value::String(label.into_owned()));
        }

        let features = set(values(prop, IcalParamKind::Feature)
            .iter()
            .map(|feature| feature.to_lowercase())
            .collect());

        if !features.is_empty() {
            object.insert("features".to_owned(), Value::Object(features));
        }

        let pointer = format!("virtualLocations/{key}");
        self.virtual_locations.insert(key, Value::Object(object));
        self.hatch.note(
            &pointer,
            prop,
            &[
                IcalParamKind::Label,
                IcalParamKind::Feature,
                IcalParamKind::Value,
            ],
        );
    }

    /// `ATTACH`, `IMAGE` and `LINK` are Links, told apart by their relation
    /// (RFC 8984 1.4.11).
    pub(super) fn link(&mut self, prop: &IcalProp<'_>, rel: Option<&str>) {
        let Some(href) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let key = key(&self.links, prop);
        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Link".to_owned()));
        object.insert("href".to_owned(), Value::String(href));

        if let Some(rel) = rel {
            object.insert("rel".to_owned(), Value::String(rel.to_lowercase()));
        }

        if let Some(media) = param(prop, IcalParamKind::FmtType) {
            object.insert("contentType".to_owned(), Value::String(media.into_owned()));
        }

        if let Some(display) = param(prop, IcalParamKind::Display) {
            object.insert("display".to_owned(), Value::String(display.to_lowercase()));
        }

        if let Some(label) = param(prop, IcalParamKind::Label) {
            object.insert("title".to_owned(), Value::String(label.into_owned()));
        }

        let pointer = format!("links/{key}");
        self.links.insert(key, Value::Object(object));
        self.hatch.note(
            &pointer,
            prop,
            &[
                IcalParamKind::FmtType,
                IcalParamKind::Display,
                IcalParamKind::Label,
                IcalParamKind::LinkRel,
            ],
        );
    }

    /// A `VLOCATION` component as a Location (draft 2.2.4).
    pub(super) fn vlocation(&mut self, component: &IcalComponent<'_>) {
        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Location".to_owned()));

        let mut hatch = IcalHatch::new("vlocation");
        let mut types = Map::new();

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
                (IcalPropKind::Name, Some(name)) => {
                    object.insert("name".to_owned(), Value::String(name));
                    hatch.note("name", prop, &[]);
                }
                (IcalPropKind::Description, Some(description)) => {
                    object.insert("description".to_owned(), Value::String(description));
                    hatch.note("description", prop, &[]);
                }
                (IcalPropKind::LocationType, _) => {
                    types.extend(set(list(prop)));
                    hatch.note("locationTypes", prop, &[]);
                }
                (IcalPropKind::Url, Some(uri)) => {
                    object.insert("coordinates".to_owned(), Value::String(uri));
                    hatch.note("coordinates", prop, &[]);
                }
                _ => hatch.keep(prop),
            }
        }

        for child in &component.components {
            hatch.keep_component(child);
        }

        if !types.is_empty() {
            object.insert("locationTypes".to_owned(), Value::Object(types));
        }

        if let Some(hatch) = hatch.into_value() {
            object.insert("iCalendar".to_owned(), hatch);
        }

        let key = component_key(&self.locations, component);
        self.locations.insert(key, Value::Object(object));
    }
}
