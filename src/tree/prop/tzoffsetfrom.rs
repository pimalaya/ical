//! # TZOFFSETFROM lens
//!
//! The `TZOFFSETFROM` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::utc_offset::IcalUtcOffset,
    version::IcalVersion,
};

/// The `TZOFFSETFROM` property lens.
#[allow(non_camel_case_types)]
pub struct TZOFFSETFROM;

impl IcalPropLens for TZOFFSETFROM {
    type Target<'v> = IcalUtcOffset<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for TZOFFSETFROM {
    const KIND: IcalPropKind = IcalPropKind::TzOffsetFrom;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::UtcOffset]
    }
}
