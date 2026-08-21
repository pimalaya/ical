//! # TZ lens
//!
//! The `TZ` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::utc_offset::IcalUtcOffset,
    version::IcalVersion,
};

/// The `TZ` property lens.
#[allow(non_camel_case_types)]
pub struct TZ;

impl IcalPropLens for TZ {
    type Target<'v> = IcalUtcOffset<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for TZ {
    const KIND: IcalPropKind = IcalPropKind::Tz;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::UtcOffset]
    }
}
