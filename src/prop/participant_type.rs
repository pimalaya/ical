//! # PARTICIPANT-TYPE
//!
//! The `PARTICIPANT-TYPE` property: what part a `PARTICIPANT` plays (RFC 9073
//! 6.2).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `PARTICIPANT-TYPE` property marker.
#[allow(non_camel_case_types)]
pub struct PARTICIPANT_TYPE;

impl IcalPropSpec for PARTICIPANT_TYPE {
    const KIND: IcalPropKind = IcalPropKind::ParticipantType;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
