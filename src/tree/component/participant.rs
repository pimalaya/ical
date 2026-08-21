//! # PARTICIPANT component lens
//!
//! The `PARTICIPANT` component lens.

use crate::{
    component::IcalComponentKind,
    tree::component::{lens::IcalComponentLens, spec::IcalComponentSpec},
};

/// The `PARTICIPANT` component lens.
#[allow(non_camel_case_types)]
pub struct PARTICIPANT;

impl IcalComponentLens for PARTICIPANT {}

impl IcalComponentSpec for PARTICIPANT {
    const KIND: IcalComponentKind = IcalComponentKind::Participant;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[IcalComponentKind::VLocation, IcalComponentKind::VResource]
    }
}
