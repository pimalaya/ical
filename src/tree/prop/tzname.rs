//! # TZNAME lens
//!
//! Reading and editing the `TZNAME` property in place: it decodes as an
//! [`IcalText`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`TZNAME`].

use crate::{
    prop::tzname::TZNAME,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::text::IcalText,
};

impl IcalPropLens for TZNAME {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
