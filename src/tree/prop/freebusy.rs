//! # FREEBUSY lens
//!
//! The `FREEBUSY` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::period::IcalPeriod,
    version::IcalVersion,
};

/// The `FREEBUSY` property lens.
#[allow(non_camel_case_types)]
pub struct FREEBUSY;

impl IcalPropLens for FREEBUSY {
    type Target<'v> = IcalPeriod<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for FREEBUSY {
    const KIND: IcalPropKind = IcalPropKind::FreeBusy;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Period]
    }
}
