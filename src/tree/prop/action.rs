//! # ACTION lens
//!
//! The `ACTION` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::text::IcalText,
};

/// The `ACTION` property lens.
#[allow(non_camel_case_types)]
pub struct ACTION;

impl IcalPropLens for ACTION {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for ACTION {
    const KIND: IcalPropKind = IcalPropKind::Action;
}
