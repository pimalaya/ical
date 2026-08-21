//! # TZOFFSETTO lens
//!
//! The `TZOFFSETTO` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{cardinality::IcalPropCardinality, lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::utc_offset::IcalUtcOffset,
    version::IcalVersion,
};

/// The `TZOFFSETTO` property lens.
#[allow(non_camel_case_types)]
pub struct TZOFFSETTO;

impl IcalPropLens for TZOFFSETTO {
    type Target<'v> = IcalUtcOffset<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for TZOFFSETTO {
    const KIND: IcalPropKind = IcalPropKind::TzOffsetTo;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::UtcOffset]
    }
}
