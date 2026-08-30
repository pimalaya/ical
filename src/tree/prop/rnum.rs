//! # RNUM lens
//!
//! Reading and editing the `RNUM` property in place: it decodes as an
//! [`IcalInteger`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`RNUM`].

use crate::{
    prop::rnum::RNUM,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::integer::IcalInteger,
};

impl IcalPropLens for RNUM {
    type Target<'v> = IcalInteger<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
