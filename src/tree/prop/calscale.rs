//! # CALSCALE lens
//!
//! Reading and editing the `CALSCALE` property in place: it decodes as an
//! [`IcalText`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`CALSCALE`].

use crate::{
    prop::calscale::CALSCALE,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::text::IcalText,
};

impl IcalPropLens for CALSCALE {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
