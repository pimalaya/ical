//! # GEO lens
//!
//! Reading and editing the `GEO` property in place: it decodes as an
//! [`IcalGeo`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`GEO`].

use crate::{
    prop::geo::GEO,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::geo::IcalGeo,
};

impl IcalPropLens for GEO {
    type Target<'v> = IcalGeo<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
