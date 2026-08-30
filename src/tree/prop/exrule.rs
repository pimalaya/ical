//! # EXRULE lens
//!
//! Reading and editing the `EXRULE` property in place: it decodes as an
//! [`IcalRecur`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`EXRULE`].

use crate::{
    prop::exrule::EXRULE,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::recur::IcalRecur,
};

impl IcalPropLens for EXRULE {
    type Target<'v> = IcalRecur<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
