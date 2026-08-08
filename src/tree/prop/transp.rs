//! # TRANSP lens
//!
//! The `TRANSP` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropCardinality, IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::text::IcalText,
    version::IcalVersion,
};

/// The `TRANSP` property lens.
#[allow(non_camel_case_types)]
pub struct TRANSP;

impl IcalPropLens for TRANSP {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for TRANSP {
    const KIND: IcalPropKind = IcalPropKind::Transp;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
