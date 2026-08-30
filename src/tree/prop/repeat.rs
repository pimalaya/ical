//! # REPEAT lens
//!
//! Reading and editing the `REPEAT` property in place: it decodes as an
//! [`IcalInteger`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`REPEAT`].

use crate::{
    prop::repeat::REPEAT,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::integer::IcalInteger,
};

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
