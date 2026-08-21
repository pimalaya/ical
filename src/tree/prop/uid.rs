//! # UID lens
//!
//! The `UID` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{cardinality::IcalPropCardinality, lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::text::IcalText,
    version::IcalVersion,
};

/// The `UID` property lens.
#[allow(non_camel_case_types)]
pub struct UID;

impl IcalPropLens for UID {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for UID {
    const KIND: IcalPropKind = IcalPropKind::Uid;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
