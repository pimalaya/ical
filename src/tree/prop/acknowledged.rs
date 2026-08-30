//! # ACKNOWLEDGED lens
//!
//! Reading and editing the `ACKNOWLEDGED` property in place: it decodes as an
//! [`IcalDateTime`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`ACKNOWLEDGED`].

use crate::{
    prop::acknowledged::ACKNOWLEDGED,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::datetime::IcalDateTime,
};

impl IcalPropLens for ACKNOWLEDGED {
    type Target<'v> = IcalDateTime<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
