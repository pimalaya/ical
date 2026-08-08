//! # REPEAT lens
//!
//! The `REPEAT` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropCardinality, IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::integer::IcalInteger,
    version::IcalVersion,
};

/// The `REPEAT` property lens.
#[allow(non_camel_case_types)]
pub struct REPEAT;

impl IcalPropLens for REPEAT {
    type Target<'v> = IcalInteger<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for REPEAT {
    const KIND: IcalPropKind = IcalPropKind::Repeat;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Integer]
    }
}
