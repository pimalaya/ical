//! # LOCATION-TYPE lens
//!
//! The `LOCATION-TYPE` property lens.

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

/// The `LOCATION-TYPE` property lens.
#[allow(non_camel_case_types)]
pub struct LOCATION_TYPE;

impl IcalPropLens for LOCATION_TYPE {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for LOCATION_TYPE {
    const KIND: IcalPropKind = IcalPropKind::LocationType;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
