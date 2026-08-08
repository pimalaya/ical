//! # DALARM lens
//!
//! The `DALARM` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::text::IcalText,
    version::IcalVersion,
};

/// The `DALARM` property lens.
#[allow(non_camel_case_types)]
pub struct DALARM;

impl IcalPropLens for DALARM {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for DALARM {
    const KIND: IcalPropKind = IcalPropKind::DAlarm;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0]
    }
}
