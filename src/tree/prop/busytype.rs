//! # BUSYTYPE lens
//!
//! The `BUSYTYPE` property lens.

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

/// The `BUSYTYPE` property lens.
#[allow(non_camel_case_types)]
pub struct BUSYTYPE;

impl IcalPropLens for BUSYTYPE {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for BUSYTYPE {
    const KIND: IcalPropKind = IcalPropKind::BusyType;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
