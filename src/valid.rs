//! # Validity proof
//!
//! [`IcalValid`], the marker a check mints and nothing else can.
//!
//! Validity is a runtime predicate in this crate, never a second, stricter
//! type: a conformant calendar may still carry extensions, so a no-extension
//! type would name a useless category. What a type *can* carry is the proof
//! that a check ran and passed, which is what this is. It is a plain wrapper
//! with a private field, so the only way to hold one is to have been handed it
//! by a validator.
//!
//! Two validators mint it today, and they live at opposite ends of the crate:
//! [`Ical::validate`](crate::ical::Ical::validate) over a whole calendar, and
//! [`IcalRecurRule::validate`](crate::recur::IcalRecurRule::validate) over one
//! recurrence rule. The marker is here, in the dependency-free core, so neither
//! has to depend on the other's feature to speak the same language.

use core::ops;

/// A value that passed its validator. Only a validator can mint one, so holding
/// it is proof of conformance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IcalValid<T>(pub(crate) T);

impl<T> IcalValid<T> {
    /// Unwrap the validated value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> ops::Deref for IcalValid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
