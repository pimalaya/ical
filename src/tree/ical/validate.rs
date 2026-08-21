//! # Validation
//!
//! The "strict out" conformance check over the decoded [`Ical`] model, and the
//! [`IcalValid`] proof it mints.
//!
//! Validation walks the whole component tree and reports, against the
//! calendar's version and the [property](crate::tree::prop::spec::IcalPropSpec) and
//! [component](crate::tree::component::spec::IcalComponentSpec) specs:
//!
//! - a property the version does not define;
//! - a value of a kind the property does not take;
//! - a parameter the property does not take;
//! - a property that appears more often than it may;
//! - a property a component requires but does not carry (a `VEVENT` needs `UID`
//!   and `DTSTAMP`, a `VALARM` needs `ACTION` and `TRIGGER`, ...);
//! - a component nested where it may not be;
//! - a recurrence rule that breaks RFC 5545 3.3.10.
//!
//! Extensions always pass: validity is a runtime predicate, not a second strict
//! type, so a conformant calendar may still carry unknown components,
//! properties, parameters and value kinds.
//!
//! `validate_prop` is the per-property check, shared with the
//! [`builder`](crate::tree::ical::builder). A calendar that passes earns a
//! [`IcalValid<Ical>`], which only this module (or `TryFrom`) can mint.

use core::{error, fmt};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    ical::Ical,
    param::IcalParamKind,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    tree::{component::component_spec, prop::prop_spec},
    valid::IcalValid,
    value::IcalValueKind,
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
    /// A property carries a value of a kind its spec does not allow for the
    /// version.
    ValueKind {
        /// The offending property name.
        prop: IcalPropKind,
        /// The kind the value actually has.
        kind: IcalValueKind,
    },
    /// A property carries a known parameter its spec does not allow for the
    /// version. An extension parameter always passes.
    ParamNotAllowed {
        /// The offending property name.
        prop: IcalPropKind,
        /// The parameter that may not appear on it.
        param: IcalParamKind,
    },
    /// A property appears more times than its cardinality permits.
    ///
    /// Only the "too many" direction: a property that is absent when it should
    /// be there is a [`MissingProp`](Self::MissingProp), since whether a
    /// property is required depends on the component it sits in and the
    /// cardinality does not.
    TooMany {
        /// The component name.
        component: String,
        /// The repeated property.
        prop: IcalPropKind,
        /// How many times it appears.
        count: usize,
    },
    /// A component nests a child its spec does not allow.
    Nesting {
        /// The parent component name.
        parent: String,
        /// The child it may not hold.
        child: IcalComponentKind,
    },
    /// A recurrence rule breaks one of the RFC 5545 3.3.10 constraints.
    Rule {
        /// The property carrying the rule (`RRULE` or `EXRULE`).
        prop: IcalPropKind,
        /// What is wrong with it.
        problem: crate::recur::validate::IcalRecurRuleProblem,
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
            Self::ValueKind { prop, kind } => {
                write!(
                    f,
                    "Property `{}` does not take a {} value",
                    &**prop, &**kind
                )
            }
            Self::ParamNotAllowed { prop, param } => {
                write!(
                    f,
                    "Property `{}` does not take the `{}` parameter",
                    &**prop, &**param
                )
            }
            Self::TooMany {
                component,
                prop,
                count,
            } => {
                write!(
                    f,
                    "Component `{component}` carries property `{}` {count} times",
                    &**prop
                )
            }
            Self::Nesting { parent, child } => {
                write!(
                    f,
                    "Component `{parent}` does not nest a `{}` component",
                    &**child
                )
            }
            Self::Rule { prop, problem } => {
                write!(
                    f,
                    "Property `{}` carries an invalid rule: {problem}",
                    &**prop
                )
            }
        }
    }
}

impl error::Error for IcalValidateError {}

impl Ical<'_> {
    /// Validate the whole calendar, returning an [`IcalValid`] proof or every
    /// conformance failure found.
    pub fn validate(self) -> Result<IcalValid<Self>, Vec<IcalValidateError>> {
        let mut errors = Vec::new();

        for prop in &self.props {
            validate_prop(prop, self.version, &mut errors);
        }
        // NOTE: The calendar envelope requires PRODID (VERSION is the
        // hoisted-out indicator, always present in the model).
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
            Ok(IcalValid(self))
        } else {
            Err(errors)
        }
    }
}

/// Validate one component (recursively): its properties, its required-property
/// set, how many times each property appears, what it nests, and its nested
/// components.
fn validate_component(
    component: &IcalComponent<'_>,
    version: IcalVersion,
    errors: &mut Vec<IcalValidateError>,
) {
    for prop in &component.props {
        validate_prop(prop, version, errors);
    }

    check_cardinality(&component.name, &component.props, version, errors);

    if let IcalComponentName::Kind(kind) = component.name {
        check_required(kind, &component.name, &component.props, errors);
        check_nesting(kind, &component.name, &component.components, errors);
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
/// [`builder`](crate::tree::ical::builder).
///
/// Unknown (extension) properties, parameters and value kinds always pass:
/// validity is a runtime predicate over the *known* vocabulary, and an
/// extension is outside it by definition. A known property must exist in the
/// calendar's version, take a value of a kind its spec allows there, and carry
/// only parameters that spec allows there. A recurrence value is checked
/// against RFC 5545 3.3.10 as well.
pub(crate) fn validate_prop(
    prop: &IcalProp<'_>,
    version: IcalVersion,
    errors: &mut Vec<IcalValidateError>,
) {
    let IcalPropName::Kind(kind) = prop.name else {
        return;
    };

    let spec = prop_spec(kind);

    if !(spec.allowed_versions)().contains(&version) {
        errors.push(IcalValidateError::PropVersion {
            prop: (*kind).to_string(),
            version,
        });
    }

    if let Some(value) = prop.value.kind()
        && !(spec.allowed_values)(version).contains(&value)
    {
        errors.push(IcalValidateError::ValueKind {
            prop: kind,
            kind: value,
        });
    }

    let allowed_params = (spec.allowed_params)(version);
    for param in &prop.params {
        if let Some(param) = param.kind()
            && !allowed_params.contains(&param)
        {
            errors.push(IcalValidateError::ParamNotAllowed { prop: kind, param });
        }
    }

    validate_rule(kind, prop, errors);
}

/// Check the rule a `RRULE` or `EXRULE` carries against RFC 5545 3.3.10.
///
/// A rule the typed layer cannot even read is left alone: parsing is liberal,
/// and an unreadable rule is a parse-level fact, not a conformance one.
fn validate_rule(kind: IcalPropKind, prop: &IcalProp<'_>, errors: &mut Vec<IcalValidateError>) {
    use crate::{recur::IcalRecurRule, value::IcalValue};

    if !matches!(kind, IcalPropKind::RRule | IcalPropKind::ExRule) {
        return;
    }

    let IcalValue::Recur(recur) = &prop.value else {
        return;
    };

    let Ok(rule) = IcalRecurRule::parse(&recur.0) else {
        return;
    };

    errors.extend(
        rule.problems()
            .into_iter()
            .map(|problem| IcalValidateError::Rule {
                prop: kind,
                problem,
            }),
    );
}

/// Push a [`TooMany`](IcalValidateError::TooMany) for every property that
/// appears more times than its spec permits.
///
/// Only the "too many" direction: whether a property is *required* depends on
/// the component it sits in, which the per-property cardinality does not know,
/// so absence stays [`check_required`]'s job.
fn check_cardinality(
    name: &str,
    props: &[IcalProp<'_>],
    version: IcalVersion,
    errors: &mut Vec<IcalValidateError>,
) {
    use crate::tree::prop::cardinality::IcalPropCardinality::{AtMostOne, ExactlyOne};

    let mut seen: Vec<(IcalPropKind, usize)> = Vec::new();

    for prop in props {
        let IcalPropName::Kind(kind) = prop.name else {
            continue;
        };

        match seen.iter_mut().find(|(held, _)| *held == kind) {
            Some((_, count)) => *count += 1,
            None => seen.push((kind, 1)),
        }
    }

    for (kind, count) in seen {
        if count > 1
            && matches!(
                (prop_spec(kind).cardinality)(version),
                ExactlyOne | AtMostOne
            )
        {
            errors.push(IcalValidateError::TooMany {
                component: name.to_string(),
                prop: kind,
                count,
            });
        }
    }
}

/// Push a [`Nesting`](IcalValidateError::Nesting) for every child a component
/// may not hold. An unknown child component always passes.
fn check_nesting(
    kind: IcalComponentKind,
    name: &str,
    children: &[IcalComponent<'_>],
    errors: &mut Vec<IcalValidateError>,
) {
    let allowed = (component_spec(kind).allowed_children)();

    for child in children {
        if let IcalComponentName::Kind(child_kind) = child.name
            && !allowed.contains(&child_kind)
        {
            errors.push(IcalValidateError::Nesting {
                parent: name.to_string(),
                child: child_kind,
            });
        }
    }
}

impl<'a> TryFrom<Ical<'a>> for IcalValid<Ical<'a>> {
    type Error = Vec<IcalValidateError>;

    fn try_from(cal: Ical<'a>) -> Result<Self, Self::Error> {
        cal.validate()
    }
}

impl<'a> From<IcalValid<Ical<'a>>> for crate::tree::cst::IcalCst<'static> {
    fn from(valid: IcalValid<Ical<'a>>) -> Self {
        valid.into_inner().encode()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::{
        component::{IcalComponent, IcalComponentKind},
        ical::Ical,
        param::IcalParamKind,
        prop::{IcalProp, IcalPropKind},
        tree::ical::validate::IcalValidateError,
        value::{IcalValue, IcalValueKind, datetime::IcalDateTime, text::IcalText},
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
                // NOTE: Missing UID and DTSTAMP.
                props: vec![],
                components: vec![],
            }],
        };
        let errors = cal.validate().unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    /// A conformant calendar wrapping one `VEVENT` built from `props`.
    fn around(props: vec::Vec<IcalProp<'static>>) -> Ical<'static> {
        let mut event = vec![
            prop(IcalPropKind::Uid, IcalValue::Text(IcalText("1".into()))),
            prop(
                IcalPropKind::DtStamp,
                IcalValue::DateTime(IcalDateTime("20260101T000000Z".into())),
            ),
        ];
        event.extend(props);

        Ical {
            version: IcalVersion::V2_0,
            props: vec![prop(
                IcalPropKind::ProdId,
                IcalValue::Text(IcalText("-//x//EN".into())),
            )],
            components: vec![IcalComponent {
                name: IcalComponentKind::VEvent.into(),
                props: event,
                components: vec![],
            }],
        }
    }

    #[test]
    fn flags_a_value_of_the_wrong_kind() {
        // NOTE: SUMMARY is text, not a date-time.
        let cal = around(vec![prop(
            IcalPropKind::Summary,
            IcalValue::DateTime(IcalDateTime("20260101T000000Z".into())),
        )]);

        assert_eq!(
            cal.validate().unwrap_err(),
            [IcalValidateError::ValueKind {
                prop: IcalPropKind::Summary,
                kind: IcalValueKind::DateTime,
            }]
        );
    }

    #[test]
    fn passes_an_extension_value_kind() {
        // NOTE: An unknown value has no kind to check, so it cannot be the
        // wrong one.
        let cal = around(vec![IcalProp {
            name: "X-THING".into(),
            params: vec![],
            value: IcalValue::DateTime(IcalDateTime("20260101T000000Z".into())),
        }]);

        assert!(cal.validate().is_ok());
    }

    #[test]
    fn flags_a_parameter_the_property_does_not_take() {
        use crate::param::IcalParam;

        // NOTE: PARTSTAT belongs on ATTENDEE, not on SUMMARY.
        let cal = around(vec![IcalProp {
            name: IcalPropKind::Summary.into(),
            params: vec![IcalParam::PartStat("ACCEPTED".into())],
            value: IcalValue::Text(IcalText("Lunch".into())),
        }]);

        assert_eq!(
            cal.validate().unwrap_err(),
            [IcalValidateError::ParamNotAllowed {
                prop: IcalPropKind::Summary,
                param: IcalParamKind::PartStat,
            }]
        );
    }

    #[test]
    fn passes_an_extension_parameter() {
        use crate::param::IcalParam;

        let cal = around(vec![IcalProp {
            name: IcalPropKind::Summary.into(),
            params: vec![IcalParam::Unknown {
                name: "X-THING".into(),
                values: vec!["1".into()],
            }],
            value: IcalValue::Text(IcalText("Lunch".into())),
        }]);

        assert!(cal.validate().is_ok());
    }

    #[test]
    fn flags_a_single_valued_property_that_repeats() {
        let cal = around(vec![
            prop(IcalPropKind::Summary, IcalValue::Text(IcalText("a".into()))),
            prop(IcalPropKind::Summary, IcalValue::Text(IcalText("b".into()))),
        ]);

        assert_eq!(
            cal.validate().unwrap_err(),
            [IcalValidateError::TooMany {
                component: "VEVENT".into(),
                prop: IcalPropKind::Summary,
                count: 2,
            }]
        );
    }

    #[test]
    fn passes_a_repeatable_property_that_repeats() {
        let cal = around(vec![
            prop(
                IcalPropKind::Comment,
                IcalValue::Text(IcalText("one".into())),
            ),
            prop(
                IcalPropKind::Comment,
                IcalValue::Text(IcalText("two".into())),
            ),
        ]);

        assert!(cal.validate().is_ok());
    }

    #[test]
    fn flags_a_component_nested_where_it_may_not_be() {
        // NOTE: A VTIMEZONE belongs to the calendar, not to an event.
        let mut cal = around(vec![]);
        cal.components[0].components.push(IcalComponent {
            name: IcalComponentKind::VTimezone.into(),
            props: vec![prop(
                IcalPropKind::TzId,
                IcalValue::Text(IcalText("Europe/Paris".into())),
            )],
            components: vec![],
        });

        let errors = cal.validate().unwrap_err();
        assert!(errors.contains(&IcalValidateError::Nesting {
            parent: "VEVENT".into(),
            child: IcalComponentKind::VTimezone,
        }));
    }

    #[test]
    fn passes_an_extension_component() {
        let mut cal = around(vec![]);
        cal.components[0].components.push(IcalComponent {
            name: "X-THING".into(),
            props: vec![],
            components: vec![],
        });

        assert!(cal.validate().is_ok());
    }

    #[test]
    fn flags_a_rule_the_rfc_forbids() {
        use crate::{
            recur::{IcalRecurFreq, validate::IcalRecurPart, validate::IcalRecurRuleProblem},
            value::recur::IcalRecur,
        };

        let cal = around(vec![prop(
            IcalPropKind::RRule,
            IcalValue::Recur(IcalRecur("FREQ=MONTHLY;BYWEEKNO=3".into())),
        )]);

        assert_eq!(
            cal.validate().unwrap_err(),
            [IcalValidateError::Rule {
                prop: IcalPropKind::RRule,
                problem: IcalRecurRuleProblem::PartFreq {
                    part: IcalRecurPart::ByWeekNo,
                    freq: IcalRecurFreq::Monthly,
                },
            }]
        );
    }
}
