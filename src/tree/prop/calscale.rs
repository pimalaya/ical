//! # CALSCALE lens
//!
//! The `CALSCALE` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::text::IcalText,
};

/// The `CALSCALE` property lens.
#[allow(non_camel_case_types)]
pub struct CALSCALE;

impl IcalPropLens for CALSCALE {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for CALSCALE {
    const KIND: IcalPropKind = IcalPropKind::CalScale;
}
