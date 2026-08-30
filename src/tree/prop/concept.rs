//! # CONCEPT lens
//!
//! Reading and editing the `CONCEPT` property in place: it decodes as an
//! [`IcalUri`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`CONCEPT`].

use crate::{
    prop::concept::CONCEPT,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::uri::IcalUri,
};

impl IcalPropLens for CONCEPT {
    type Target<'v> = IcalUri<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
