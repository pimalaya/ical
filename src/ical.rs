//! # Calendar
//!
//! The decoded calendar: a version plus the `VCALENDAR`'s properties and nested
//! components.
//!
//! [`Ical`] is the top of the decoded model, the semantic counterpart of a
//! whole [`IcalCst`](crate::tree::cst::IcalCst). It is a `VCALENDAR` with its
//! [`version`](Ical::version) hoisted out of the property list (the `VERSION`
//! line is the envelope indicator, not a free property), its remaining
//! calendar-level [`props`](Ical::props) (`PRODID`, `CALSCALE`, `METHOD`, ...),
//! and its nested [`components`](Ical::components) (`VEVENT`, `VTODO`,
//! `VTIMEZONE`, ...). Each nested component is a recursive
//! [`IcalComponent`].
//!
//! [`IcalComponent`]: crate::component::IcalComponent
//!
//! Build a calendar directly from its public fields; strict, spec-checked
//! construction and conformance checking live in the syntax layer
//! ([`validate`](crate::tree::ical::validate)). This module is pure model: no
//! dependency on [`crate::tree`].

use alloc::vec::Vec;

use crate::{component::IcalComponent, prop::IcalProp, version::IcalVersion};

/// A decoded calendar: a `VCALENDAR` with its version hoisted out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ical<'a> {
    /// The calendar version, read from the `VERSION` property.
    pub version: IcalVersion,
    /// The calendar-level properties (`PRODID`, `CALSCALE`, `METHOD`, ...),
    /// excluding `VERSION`.
    pub props: Vec<IcalProp<'a>>,
    /// The components nested in the calendar (`VEVENT`, `VTODO`, `VTIMEZONE`,
    /// ...), in source order.
    pub components: Vec<IcalComponent<'a>>,
}

impl Ical<'_> {
    /// The same calendar with every borrow replaced by an allocation, so it
    /// outlives the bytes it was decoded from. See
    /// [`IcalValue::into_owned`](crate::value::IcalValue::into_owned).
    pub fn into_owned(self) -> Ical<'static> {
        Ical {
            version: self.version,
            props: self.props.into_iter().map(IcalProp::into_owned).collect(),
            components: self
                .components
                .into_iter()
                .map(IcalComponent::into_owned)
                .collect(),
        }
    }
}
