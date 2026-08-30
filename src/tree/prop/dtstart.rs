//! # DTSTART lens
//!
//! Reading and editing the `DTSTART` property in place: it decodes as an
//! [`IcalDateTime`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`DTSTART`].

use crate::{
    prop::dtstart::DTSTART,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::datetime::IcalDateTime,
};

impl IcalPropLens for DTSTART {
    type Target<'v> = IcalDateTime<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
