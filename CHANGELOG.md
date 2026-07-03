# Changelog

All notable changes to this project are documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

## [Unreleased]

### Added

- Added the closed kind vocabularies (IcalComponentKind, IcalPropKind, IcalParamKind, IcalValueKind, IcalVersion), each with FromStr and Deref<str> for its wire name, plus IcalValue::kind and IcalParam::kind.
- Added IcalPropName (a known IcalPropKind or a verbatim unknown name) and IcalComponentName (a known IcalComponentKind or a verbatim unknown name); an unrecognised or missing calendar version normalises to IcalVersion::V2_0 in the decoded model (byte-faithful round-tripping stays on the syntax tree).
- Added the per-property IcalPropSpec contract (allowed_versions, cardinality, allowed_values, allowed_params and the in-force value) on the lens markers, with IcalPropCardinality, filled per RFC 5545 and its extensions, plus IcalPropKind::ALL.
- Added the per-component IcalComponentSpec contract (allowed versions, allowed parent and child components, required and optional properties) on the component lens markers, filled per RFC 5545 (VCALENDAR requires VERSION and PRODID, VEVENT requires DTSTAMP and UID, VALARM requires ACTION and TRIGGER, ...).
- Added Ical::validate, an RFC 5545 conformance check over the decoded model that walks the component tree (per-version property existence, value kind, version-aware parameters, cardinality including required-but-absent, and component nesting) and permits extensions; Valid<T>, a marker only validation can mint (TryFrom<Ical> for Valid<Ical>); and From conversions Ical -> IcalCst and Valid<Ical> -> IcalCst.
- Added IcalPropBuilder, a version-aware, spec-driven builder for strict construction: it pins the property name and reuses the per-property validation, rejecting (via Result) a disallowed value kind or known parameter while allowing extension parameters.
- Added the version-agnostic decoded model (parser-free, always available): the Ical aggregate (a version plus VCALENDAR-level properties and nested components), the recursive IcalComponent (name, properties and nested components), IcalProp (name, parameters and one value), the open IcalParam and IcalValue payload enums (each with an Unknown arm), and the value types IcalText/IcalTextList, IcalBinary, IcalBoolean, IcalInteger, IcalFloat, IcalDate/IcalDateTime/IcalTime, IcalDuration, IcalPeriod, IcalUtcOffset, IcalCalAddress, IcalUri, IcalGeo, IcalRecur and IcalRequestStatus.
- Added the byte-faithful syntax tree (parser feature, on by default): IcalCst parses bytes or text into a recursive component tree that reproduces the wire exactly, decodes onto the model, encodes back to a canonical tree, and edits one property in place through per-property lenses and byte-preserving cursors; to_bytes is the byte-faithful serializer, while Display / to_string is a lossy-for-non-UTF-8 convenience.
- Added raw-byte value handling: a property value is kept as bytes so a value in a foreign CHARSET survives byte for byte, while a name or parameter must be UTF-8 (a non-UTF-8 name or parameter is a parse error), plus a byte hatch (IcalValueCursor::bytes / set_bytes).
- Added multi-calendar parsing: IcalCst::parse_many iterates every VCALENDAR in a file.
- Added typed component access: IcalCst::component / component_mut / components walk the nested tree by lens marker (VEVENT, VTODO, VALARM, VTIMEZONE, ...).
- Added opt-in content-decoding features, each backed by a no_std crate: quoted-printable (=XX octets, IcalValueCursor::quoted_printable), base64 (inline binary values, IcalBinary::decode_base64), and encoding (foreign CHARSET transcoding via encoding_rs, IcalValueCursor::charset).
