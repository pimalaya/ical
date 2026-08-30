//! # Component spec
//!
//! The per-component contract on the component markers (the child components it
//! may nest and the properties it requires), and the runtime vtable that
//! bridges the open [`IcalComponentKind`] back to those static impls.

use crate::{
    component::{
        IcalComponentKind, available, daylight, participant, standard, valarm, vavailability,
        vcalendar, vevent, vfreebusy, vjournal, vlocation, vresource, vtimezone, vtodo,
    },
    prop::IcalPropKind,
};

/// The per-component contract: the child components it may nest and the
/// properties it requires. Implemented on the zero-sized component markers; the
/// defaults are the permissive empty sets, so a component overrides only where
/// it constrains.
pub trait IcalComponentSpec {
    /// The component this spec describes.
    const KIND: IcalComponentKind;

    /// The child components this one may directly nest.
    fn allowed_children() -> &'static [IcalComponentKind] {
        &[]
    }

    /// The properties this component requires.
    fn required_props() -> &'static [IcalPropKind] {
        &[]
    }
}

/// The spec of a component as function pointers, the runtime bridge from the
/// open [`IcalComponentKind`] back to the static per-marker impls.
#[allow(dead_code)]
pub(crate) struct IcalComponentSpecFns {
    /// The component this spec describes, so the dispatch can be checked
    /// against itself.
    pub kind: IcalComponentKind,
    /// See [`IcalComponentSpec::allowed_children`].
    pub allowed_children: fn() -> &'static [IcalComponentKind],
    /// See [`IcalComponentSpec::required_props`].
    pub required_props: fn() -> &'static [IcalPropKind],
}

/// Collect the spec function pointers of a marker type.
fn spec_fns<C: IcalComponentSpec>() -> IcalComponentSpecFns {
    IcalComponentSpecFns {
        kind: C::KIND,
        allowed_children: C::allowed_children,
        required_props: C::required_props,
    }
}

/// Dispatch a component kind onto its marker spec.
pub(crate) fn component_spec(component: IcalComponentKind) -> IcalComponentSpecFns {
    use IcalComponentKind::*;

    match component {
        VCalendar => spec_fns::<vcalendar::VCALENDAR>(),
        VEvent => spec_fns::<vevent::VEVENT>(),
        VTodo => spec_fns::<vtodo::VTODO>(),
        VJournal => spec_fns::<vjournal::VJOURNAL>(),
        VFreeBusy => spec_fns::<vfreebusy::VFREEBUSY>(),
        VTimezone => spec_fns::<vtimezone::VTIMEZONE>(),
        Standard => spec_fns::<standard::STANDARD>(),
        Daylight => spec_fns::<daylight::DAYLIGHT>(),
        VAlarm => spec_fns::<valarm::VALARM>(),
        Participant => spec_fns::<participant::PARTICIPANT>(),
        VLocation => spec_fns::<vlocation::VLOCATION>(),
        VResource => spec_fns::<vresource::VRESOURCE>(),
        VAvailability => spec_fns::<vavailability::VAVAILABILITY>(),
        Available => spec_fns::<available::AVAILABLE>(),
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::component::{IcalComponentKind, spec::component_spec};

    #[test]
    fn dispatches_every_component_onto_its_own_marker() {
        for kind in IcalComponentKind::ALL {
            assert_eq!(component_spec(kind).kind, kind, "{}", &*kind);
        }
    }

    #[test]
    fn no_component_requires_one_property_twice() {
        for kind in IcalComponentKind::ALL {
            let spec = component_spec(kind);

            let mut required: Vec<&str> = (spec.required_props)().iter().map(|p| &**p).collect();
            let count = required.len();
            required.sort_unstable();
            required.dedup();

            assert_eq!(
                required.len(),
                count,
                "{} requires a property twice",
                &*kind
            );
        }
    }
}
