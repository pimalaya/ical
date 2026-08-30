//! # Places and links
//!
//! A Location, virtual location or Link read back as the property or
//! component iCalendar carries it in.

use alloc::{
    borrow::{Cow, ToOwned},
    format,
    string::String,
    vec::Vec,
};

use serde_json::{Map, Value};

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    jscalendar::import::{keyed, keys, named, plain, text_prop},
    param::IcalParam,
    prop::{IcalProp, IcalPropKind},
    value::{IcalValue, geo::IcalGeo, text::IcalTextList, uri::IcalUri},
};

/// A Link as the property it came from (`ATTACH` unless recorded otherwise).
pub(super) fn link(
    hatch: Option<&Map<String, Value>>,
    component: &str,
    key: &str,
    link: &Value,
) -> IcalProp<'static> {
    let pointer = format!("links/{key}");
    let href = link.get("href").and_then(Value::as_str).unwrap_or_default();

    let mut prop = text_prop(
        named(hatch, component, &pointer, IcalPropKind::Attach),
        href.to_owned(),
    );

    prop.value = IcalValue::Uri(IcalUri(Cow::Owned(href.to_owned())));

    if let Some(media) = link.get("contentType").and_then(Value::as_str) {
        prop.params
            .push(IcalParam::FmtType(Cow::Owned(media.to_owned())));
    }

    if let Some(display) = link.get("display").and_then(Value::as_str) {
        prop.params
            .push(IcalParam::Display(Cow::Owned(display.to_ascii_uppercase())));
    }

    if let Some(title) = link.get("title").and_then(Value::as_str) {
        prop.params
            .push(IcalParam::Label(Cow::Owned(title.to_owned())));
    }

    // NOTE: Only a LINK words its relation; an IMAGE's is always `icon` and an
    // ATTACH has none, so writing one back would invent a parameter.
    if let Some(rel) = link.get("rel").and_then(Value::as_str)
        && prop.name.eq_ignore_ascii_case("LINK")
    {
        prop.params
            .push(IcalParam::LinkRel(Cow::Owned(rel.to_ascii_uppercase())));
    }

    keyed(prop, key)
}

/// A Location as a `LOCATION` or `GEO` property, or as the `VLOCATION`
/// component it came from when it says more than one property can carry.
pub(super) fn location(
    hatch: Option<&Map<String, Value>>,
    component: &str,
    key: &str,
    location: &Value,
) -> Result<IcalProp<'static>, IcalComponent<'static>> {
    let name = location.get("name").and_then(Value::as_str);
    let coordinates = location.get("coordinates").and_then(Value::as_str);
    let described =
        location.get("description").is_some() || location.get("locationTypes").is_some();

    if described || (name.is_some() && coordinates.is_some()) {
        return Err(vlocation(location, name, coordinates));
    }

    if let Some(coordinates) = coordinates {
        let pointer = format!("locations/{key}/coordinates");
        let mut prop = text_prop(
            named(hatch, component, &pointer, IcalPropKind::Geo),
            coordinates.to_owned(),
        );

        let pair = coordinates.trim_start_matches("geo:");
        let (latitude, longitude) = pair.split_once(',').unwrap_or((pair, ""));

        prop.value = IcalValue::Geo(IcalGeo {
            latitude: Cow::Owned(latitude.to_owned()),
            longitude: Cow::Owned(longitude.to_owned()),
        });

        return Ok(keyed(prop, key));
    }

    let pointer = format!("locations/{key}");
    let prop = text_prop(
        named(hatch, component, &pointer, IcalPropKind::Location),
        name.unwrap_or_default().to_owned(),
    );

    Ok(keyed(prop, key))
}

/// A Location that says more than a `LOCATION` line can, as a `VLOCATION`.
pub(super) fn vlocation(
    location: &Value,
    name: Option<&str>,
    coordinates: Option<&str>,
) -> IcalComponent<'static> {
    let mut props = Vec::new();

    if let Some(name) = name {
        props.push(plain(IcalPropKind::Name, name.to_owned()));
    }

    if let Some(description) = location.get("description").and_then(Value::as_str) {
        props.push(plain(IcalPropKind::Description, description.to_owned()));
    }

    if let Some(coordinates) = coordinates {
        props.push(plain(IcalPropKind::Url, coordinates.to_owned()));
    }

    let types: Vec<Cow<'static, str>> = keys(location.get("locationTypes").unwrap_or(&Value::Null))
        .map(Cow::Owned)
        .collect();

    if !types.is_empty() {
        let mut prop = plain(IcalPropKind::LocationType, String::new());
        prop.value = IcalValue::TextList(IcalTextList(types));
        props.push(prop);
    }

    IcalComponent {
        name: IcalComponentName::Kind(IcalComponentKind::VLocation),
        props,
        components: Vec::new(),
    }
}

/// A VirtualLocation as a `CONFERENCE` property.
pub(super) fn conference(
    hatch: Option<&Map<String, Value>>,
    component: &str,
    key: &str,
    location: &Value,
) -> IcalProp<'static> {
    let pointer = format!("virtualLocations/{key}");
    let uri = location
        .get("uri")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let mut prop = text_prop(
        named(hatch, component, &pointer, IcalPropKind::Conference),
        uri.to_owned(),
    );

    prop.value = IcalValue::Uri(IcalUri(Cow::Owned(uri.to_owned())));

    if let Some(name) = location.get("name").and_then(Value::as_str) {
        prop.params
            .push(IcalParam::Label(Cow::Owned(name.to_owned())));
    }

    let features: Vec<Cow<'static, str>> = keys(location.get("features").unwrap_or(&Value::Null))
        .map(|feature| Cow::Owned(feature.to_ascii_uppercase()))
        .collect();

    if !features.is_empty() {
        prop.params.push(IcalParam::Feature(features));
    }

    keyed(prop, key)
}
