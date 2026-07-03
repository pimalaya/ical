//! # GEO lens
//!
//! The `GEO` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::geo::IcalGeo,
    version::IcalVersion,
};

/// The `GEO` property lens.
#[allow(non_camel_case_types)]
pub struct GEO;

impl IcalPropLens for GEO {
    type Target<'v> = IcalGeo<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for GEO {
    const KIND: IcalPropKind = IcalPropKind::Geo;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Geo]
    }
}
