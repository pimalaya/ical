//! # SEQUENCE lens
//!
//! The `SEQUENCE` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{cardinality::IcalPropCardinality, lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::integer::IcalInteger,
    version::IcalVersion,
};

/// The `SEQUENCE` property lens.
#[allow(non_camel_case_types)]
pub struct SEQUENCE;

impl IcalPropLens for SEQUENCE {
    type Target<'v> = IcalInteger<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for SEQUENCE {
    const KIND: IcalPropKind = IcalPropKind::Sequence;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Integer]
    }
}
