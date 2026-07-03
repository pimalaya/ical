//! # Validation
//!
//! The "strict out" conformance check over the decoded [`Ical`] model, and the
//! [`Valid`] proof it mints. Validation walks the whole component tree: each
//! component must carry the properties its
//! [`spec`](crate::tree::component::IcalComponentSpec) requires (a `VEVENT`
//! needs `UID` and `DTSTAMP`, a `VALARM` needs `ACTION` and `TRIGGER`, ...), and
//! each property must exist in the calendar's version. Extensions always pass:
//! validity is a runtime predicate, not a second strict type, so a conformant
//! calendar may still carry unknown components, properties and parameters.
//!
//! [`validate_prop`] is the per-property check, shared with the
//! [`builder`](crate::tree::ical::builder). A calendar that passes earns a
//! [`Valid<Ical>`], which only this module (or `TryFrom`) can mint.

use core::{error, fmt, ops};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    ical::Ical,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    tree::{component::component_spec, prop::prop_spec},
    version::IcalVersion,
};

/// A single conformance failure found by [`Ical::validate`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalValidateError {
    /// A property appears in a version that does not define it.
    PropVersion {
        /// The offending property name.
        prop: String,
        /// The calendar version.
        version: IcalVersion,
    },
    /// A component is missing a property its spec requires.
    MissingProp {
        /// The component name.
        component: String,
        /// The required property name.
        prop: IcalPropKind,
    },
}

impl fmt::Display for IcalValidateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PropVersion { prop, version } => {
                write!(
                    f,
                    "Property `{prop}` is not defined in version {}",
                    &**version
                )
            }
            Self::MissingProp { component, prop } => {
                write!(
                    f,
                    "Component `{component}` is missing required property `{}`",
                    &**prop
                )
            }
        }
    }
}

impl error::Error for IcalValidateError {}

impl Ical<'_> {
    /// Validate the whole calendar, returning a [`Valid`] proof or every
    /// conformance failure found.
    pub fn validate(self) -> Result<Valid<Self>, Vec<IcalValidateError>> {
        let mut errors = Vec::new();

        for prop in &self.props {
            validate_prop(prop, self.version, &mut errors);
        }
        // The calendar envelope requires PRODID (VERSION is the hoisted-out
        // indicator, always present in the model).
        check_required(
            IcalComponentKind::VCalendar,
            "VCALENDAR",
            &self.props,
            &mut errors,
        );

        for component in &self.components {
            validate_component(component, self.version, &mut errors);
        }

        if errors.is_empty() {
            Ok(Valid(self))
        } else {
            Err(errors)
        }
    }
}

/// Validate one component (recursively): its properties, its required-property
/// set, and its nested components.
fn validate_component(
    component: &IcalComponent<'_>,
    version: IcalVersion,
    errors: &mut Vec<IcalValidateError>,
) {
    for prop in &component.props {
        validate_prop(prop, version, errors);
    }

    if let IcalComponentName::Kind(kind) = component.name {
        check_required(kind, &component.name, &component.props, errors);
    }

    for child in &component.components {
        validate_component(child, version, errors);
    }
}

/// Push a [`MissingProp`](IcalValidateError::MissingProp) for every property a
/// component of `kind` requires but does not carry.
fn check_required(
    kind: IcalComponentKind,
    name: &str,
    props: &[IcalProp<'_>],
    errors: &mut Vec<IcalValidateError>,
) {
    for &required in (component_spec(kind).required_props)() {
        let present = props
            .iter()
            .any(|prop| matches!(prop.name, IcalPropName::Kind(k) if k == required));
        if !present {
            errors.push(IcalValidateError::MissingProp {
                component: name.to_string(),
                prop: required,
            });
        }
    }
}

/// The per-property check, shared by [`Ical::validate`] and the
/// [`builder`](crate::tree::ical::builder). Unknown (extension) properties
/// always pass; a known property must exist in the calendar's version.
pub(crate) fn validate_prop(
    prop: &IcalProp<'_>,
    version: IcalVersion,
    errors: &mut Vec<IcalValidateError>,
) {
    let IcalPropName::Kind(kind) = prop.name else {
        return;
    };

    if !(prop_spec(kind).allowed_versions)().contains(&version) {
        errors.push(IcalValidateError::PropVersion {
            prop: (*kind).to_string(),
            version,
        });
    }
}

/// A value that passed [`validate`](Ical::validate). Only validation (or
/// [`TryFrom`]) can mint it, so holding one is proof of conformance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Valid<T>(T);

impl<T> Valid<T> {
    /// Unwrap the validated value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> ops::Deref for Valid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TryFrom<Ical<'a>> for Valid<Ical<'a>> {
    type Error = Vec<IcalValidateError>;

    fn try_from(cal: Ical<'a>) -> Result<Self, Self::Error> {
        cal.validate()
    }
}

impl<'a> From<Valid<Ical<'a>>> for crate::tree::cst::IcalCst<'static> {
    fn from(valid: Valid<Ical<'a>>) -> Self {
        valid.into_inner().encode()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::{
        component::{IcalComponent, IcalComponentKind},
        ical::Ical,
        prop::{IcalProp, IcalPropKind},
        value::{IcalValue, datetime::IcalDateTime, text::IcalText},
        version::IcalVersion,
    };

    fn prop(kind: IcalPropKind, value: IcalValue<'static>) -> IcalProp<'static> {
        IcalProp {
            name: kind.into(),
            params: vec![],
            value,
        }
    }

    #[test]
    fn accepts_a_conformant_calendar() {
        let cal = Ical {
            version: IcalVersion::V2_0,
            props: vec![prop(
                IcalPropKind::ProdId,
                IcalValue::Text(IcalText("-//x//EN".into())),
            )],
            components: vec![IcalComponent {
                name: IcalComponentKind::VEvent.into(),
                props: vec![
                    prop(IcalPropKind::Uid, IcalValue::Text(IcalText("1".into()))),
                    prop(
                        IcalPropKind::DtStamp,
                        IcalValue::DateTime(IcalDateTime("20260101T000000Z".into())),
                    ),
                ],
                components: vec![],
            }],
        };
        assert!(cal.validate().is_ok());
    }

    #[test]
    fn flags_a_component_missing_a_required_property() {
        let cal = Ical {
            version: IcalVersion::V2_0,
            props: vec![prop(
                IcalPropKind::ProdId,
                IcalValue::Text(IcalText("-//x//EN".into())),
            )],
            components: vec![IcalComponent {
                name: IcalComponentKind::VEvent.into(),
                // Missing UID and DTSTAMP.
                props: vec![],
                components: vec![],
            }],
        };
        let errors = cal.validate().unwrap_err();
        assert_eq!(errors.len(), 2);
    }
}
