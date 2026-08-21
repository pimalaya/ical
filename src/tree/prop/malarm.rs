//! # MALARM lens
//!
//! The `MALARM` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::text::IcalText,
    version::IcalVersion,
};

/// The `MALARM` property lens.
#[allow(non_camel_case_types)]
pub struct MALARM;

impl IcalPropLens for MALARM {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for MALARM {
    const KIND: IcalPropKind = IcalPropKind::MAlarm;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0]
    }
}
