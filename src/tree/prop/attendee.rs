//! # ATTENDEE lens
//!
//! The `ATTENDEE` property lens.

use crate::{
    param::IcalParamKind,
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::cal_address::IcalCalAddress,
    version::IcalVersion,
};

/// The `ATTENDEE` property lens.
#[allow(non_camel_case_types)]
pub struct ATTENDEE;

impl IcalPropLens for ATTENDEE {
    type Target<'v> = IcalCalAddress<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for ATTENDEE {
    const KIND: IcalPropKind = IcalPropKind::Attendee;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::CalAddress]
    }

    /// The scheduling parameters a CalDAV server reads and writes here
    /// (RFC 6638 7) sit beside the RFC 5545 ones.
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
