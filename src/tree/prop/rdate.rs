//! # RDATE lens
//!
//! The `RDATE` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::datetime::IcalDateTime,
    version::IcalVersion,
};

/// The `RDATE` property lens.
#[allow(non_camel_case_types)]
pub struct RDATE;

impl IcalPropLens for RDATE {
    type Target<'v> = IcalDateTime<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for RDATE {
    const KIND: IcalPropKind = IcalPropKind::RDate;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
