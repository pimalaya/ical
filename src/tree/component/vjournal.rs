//! # VJOURNAL component lens
//!
//! The `VJOURNAL` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{IcalComponentLens, IcalComponentSpec},
};

/// The `VJOURNAL` component lens.
#[allow(non_camel_case_types)]
pub struct VJOURNAL;

impl IcalComponentLens for VJOURNAL {}

impl IcalComponentSpec for VJOURNAL {
    const KIND: IcalComponentKind = IcalComponentKind::VJournal;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[IcalComponentKind::Participant]
    }

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::Uid, IcalPropKind::DtStamp]
    }
}
