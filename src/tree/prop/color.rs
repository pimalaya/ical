//! # COLOR lens
//!
//! The `COLOR` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::text::IcalText,
};

/// The `COLOR` property lens.
#[allow(non_camel_case_types)]
pub struct COLOR;

impl IcalPropLens for COLOR {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for COLOR {
    const KIND: IcalPropKind = IcalPropKind::Color;
}
