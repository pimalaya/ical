//! # Property builder
//!
//! Strict, version-aware construction of a single property.
//!
//! [`IcalPropBuilder`] is the write-side counterpart of the lenses: keyed by
//! the same zero-sized markers, it carries the calendar version and accumulates
//! parameters, then emits an open [`IcalProp`]. Construction is the strict
//! half of "liberal in, strict out": the name is pinned by the marker's
//! [`IcalPropSpec`], and [`build`](IcalPropBuilder::build) runs the shared
//! per-property check ([`validate_prop`](crate::tree::ical::validate)) so a
//! known property must be defined in the calendar's version (extensions pass).
//! To emit something the spec forbids, construct the open [`IcalProp`] by hand.
//! The version is a value the builder carries, never a type parameter.
//!
//! # Example
//!
//! ```rust
//! use ical::tree::ical::builder::IcalPropBuilder;
//! use ical::tree::prop::summary::SUMMARY;
//! use ical::param::IcalParam;
//! use ical::value::IcalValue;
//! use ical::value::text::IcalText;
//! use ical::version::IcalVersion;
//! use std::borrow::Cow;
//!
//! let prop = IcalPropBuilder::<SUMMARY>::new(IcalVersion::V2_0)
//!     .param(IcalParam::Language(Cow::Borrowed("en")))
//!     .build(IcalValue::Text(IcalText(Cow::Borrowed("Lunch"))))
//!     .expect("SUMMARY accepts text with a LANGUAGE parameter");
//!
//! assert_eq!(&*prop.name, "SUMMARY");
//! ```

use core::marker::PhantomData;

use alloc::vec::Vec;

use crate::{
    param::IcalParam,
    prop::{IcalProp, IcalPropName},
    tree::{
        ical::validate::{IcalValidateError, validate_prop},
        prop::IcalPropSpec,
    },
    value::IcalValue,
    version::IcalVersion,
};

/// A version-aware builder for one property, keyed by its lens marker.
pub struct IcalPropBuilder<'a, L: IcalPropSpec> {
    /// The card version the property is built for.
    pub version: IcalVersion,
    /// The parameters accumulated so far.
    pub params: Vec<IcalParam<'a>>,
    lens: PhantomData<L>,
}

impl<'a, L: IcalPropSpec> IcalPropBuilder<'a, L> {
    /// Start a builder for the given card version.
    pub fn new(version: IcalVersion) -> Self {
        Self {
            version,
            params: Vec::new(),
            lens: PhantomData,
        }
    }

    /// Add a parameter (validated against the spec on [`build`](Self::build)).
    pub fn param(mut self, param: IcalParam<'a>) -> Self {
        self.params.push(param);
        self
    }

    /// Finish with a value, emitting the property; its name is taken from the
    /// spec. Runs the same per-property check as
    /// [`Ical::validate`](crate::ical::Ical::validate): the value kind must
    /// be allowed and every known parameter must be allowed for the version
    /// (unknown, i.e. extension, parameters pass).
    pub fn build(self, value: IcalValue<'a>) -> Result<IcalProp<'a>, Vec<IcalValidateError>> {
        let prop = IcalProp {
            name: IcalPropName::Kind(L::KIND),
            params: self.params,
            value,
        };

        let mut errors = Vec::new();

        validate_prop(&prop, self.version, &mut errors);

        if errors.is_empty() {
            Ok(prop)
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, vec};

    use crate::{
        param::IcalParam,
        tree::{ical::builder::IcalPropBuilder, prop::summary::SUMMARY},
        value::{IcalValue, text::IcalText},
        version::IcalVersion,
    };

    #[test]
    fn builds_a_property_pinning_the_name_from_the_spec() {
        let prop = IcalPropBuilder::<SUMMARY>::new(IcalVersion::V2_0)
            .param(IcalParam::Language(Cow::Borrowed("en")))
            .build(IcalValue::Text(IcalText(Cow::Borrowed("Lunch"))))
            .expect("SUMMARY takes text with a LANGUAGE param");

        assert_eq!(&*prop.name, "SUMMARY");
        assert_eq!(prop.params, vec![IcalParam::Language(Cow::Borrowed("en"))]);
        assert_eq!(
            prop.value,
            IcalValue::Text(IcalText(Cow::Borrowed("Lunch")))
        );
    }
}
