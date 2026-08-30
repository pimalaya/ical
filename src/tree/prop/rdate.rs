//! # RDATE lens
//!
//! Reading and editing the `RDATE` property in place: it decodes as an
//! [`IcalDateTimeList`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`RDATE`].

use crate::{
    prop::rdate::RDATE,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::datetime::IcalDateTimeList,
};

impl IcalPropLens for RDATE {
    type Target<'v> = IcalDateTimeList<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
