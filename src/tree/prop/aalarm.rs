//! # AALARM lens
//!
//! The `AALARM` property lens.

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

/// The `AALARM` property lens.
#[allow(non_camel_case_types)]
pub struct AALARM;

impl IcalPropLens for AALARM {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for AALARM {
    const KIND: IcalPropKind = IcalPropKind::AAlarm;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0]
    }
}
