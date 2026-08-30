//! # PARTICIPANT
//!
//! The `PARTICIPANT` component: one participant of an event, richer than an
//! `ATTENDEE` line (RFC 9073 7.1).

use crate::component::{IcalComponentKind, spec::IcalComponentSpec};

/// The `PARTICIPANT` component marker.
pub struct PARTICIPANT;

impl IcalComponentSpec for PARTICIPANT {
    const KIND: IcalComponentKind = IcalComponentKind::Participant;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[IcalComponentKind::VLocation, IcalComponentKind::VResource]
    }
}
