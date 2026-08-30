//! # SUMMARY lens
//!
//! Reading and editing the `SUMMARY` property in place: it decodes as an
//! [`IcalText`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`SUMMARY`].

use crate::{
    prop::summary::SUMMARY,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::text::IcalText,
};

impl IcalPropLens for SUMMARY {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
