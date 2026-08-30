//! # TZOFFSETTO lens
//!
//! Reading and editing the `TZOFFSETTO` property in place: it decodes as an
//! [`IcalUtcOffset`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`TZOFFSETTO`].

use crate::{
    prop::tzoffsetto::TZOFFSETTO,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::utc_offset::IcalUtcOffset,
};

impl IcalPropLens for TZOFFSETTO {
    type Target<'v> = IcalUtcOffset<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
