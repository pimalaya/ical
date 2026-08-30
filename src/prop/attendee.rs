//! # ATTENDEE
//!
//! The `ATTENDEE` property: one attendee of the component, by calendar user
//! address (RFC 5545 3.8.4.1).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `ATTENDEE` property marker.
pub struct ATTENDEE;

impl IcalPropSpec for ATTENDEE {
    const KIND: IcalPropKind = IcalPropKind::Attendee;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::CalAddress]
    }

    /// The scheduling parameters a CalDAV server reads and writes here (RFC
    /// 6638 7) sit beside the RFC 5545 ones.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Language,
            IcalParamKind::Cn,
            IcalParamKind::CuType,
            IcalParamKind::DelegatedFrom,
            IcalParamKind::DelegatedTo,
            IcalParamKind::Dir,
            IcalParamKind::Member,
            IcalParamKind::PartStat,
            IcalParamKind::Role,
            IcalParamKind::Rsvp,
            IcalParamKind::SentBy,
            IcalParamKind::Email,
            IcalParamKind::Value,
            IcalParamKind::ScheduleAgent,
            IcalParamKind::ScheduleForceSend,
            IcalParamKind::ScheduleStatus,
        ]
    }
}
