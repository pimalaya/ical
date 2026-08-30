//! # PRIORITY lens
//!
//! Reading and editing the `PRIORITY` property in place: it decodes as an
//! [`IcalInteger`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`PRIORITY`].

use crate::{
    prop::priority::PRIORITY,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::integer::IcalInteger,
};

impl IcalPropLens for PRIORITY {
    type Target<'v> = IcalInteger<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
