//! # CATEGORIES lens
//!
//! Reading and editing the `CATEGORIES` property in place: it decodes as an
//! [`IcalTextList`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`CATEGORIES`].

use crate::{
    prop::categories::CATEGORIES,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::text::IcalTextList,
};

impl IcalPropLens for CATEGORIES {
    type Target<'v> = IcalTextList<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
