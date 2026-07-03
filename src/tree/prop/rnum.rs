//! # RNUM lens
//!
//! The `RNUM` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::integer::IcalInteger,
    version::IcalVersion,
};

/// The `RNUM` property lens.
#[allow(non_camel_case_types)]
pub struct RNUM;

impl IcalPropLens for RNUM {
    type Target<'v> = IcalInteger<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for RNUM {
    const KIND: IcalPropKind = IcalPropKind::RNum;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Integer]
    }
}
