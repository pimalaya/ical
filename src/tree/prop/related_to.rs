//! # RELATED_TO lens
//!
//! The `RELATED_TO` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::text::IcalText,
};

/// The `RELATED_TO` property lens.
#[allow(non_camel_case_types)]
pub struct RELATED_TO;

impl IcalPropLens for RELATED_TO {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for RELATED_TO {
    const KIND: IcalPropKind = IcalPropKind::RelatedTo;
}
