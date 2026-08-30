//! # VJOURNAL
//!
//! The `VJOURNAL` component: a journal entry attached to a date (RFC 5545
//! 3.6.3).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `VJOURNAL` component marker.
pub struct VJOURNAL;

impl IcalComponentSpec for VJOURNAL {
    const KIND: IcalComponentKind = IcalComponentKind::VJournal;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[IcalComponentKind::Participant]
    }

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::Uid, IcalPropKind::DtStamp]
    }
}
