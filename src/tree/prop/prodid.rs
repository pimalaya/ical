//! # PRODID lens
//!
//! The `PRODID` property lens.

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

/// The `PRODID` property lens.
#[allow(non_camel_case_types)]
pub struct PRODID;

impl IcalPropLens for PRODID {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for PRODID {
    const KIND: IcalPropKind = IcalPropKind::ProdId;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
