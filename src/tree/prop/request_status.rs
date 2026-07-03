//! # REQUEST_STATUS lens
//!
//! The `REQUEST_STATUS` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::request_status::IcalRequestStatus,
    version::IcalVersion,
};

/// The `REQUEST_STATUS` property lens.
#[allow(non_camel_case_types)]
pub struct REQUEST_STATUS;

impl IcalPropLens for REQUEST_STATUS {
    type Target<'v> = IcalRequestStatus<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for REQUEST_STATUS {
    const KIND: IcalPropKind = IcalPropKind::RequestStatus;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::RequestStatus]
    }
}
