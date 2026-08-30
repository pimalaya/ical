//! # TZ lens
//!
//! Reading and editing the `TZ` property in place: it decodes as an
//! [`IcalUtcOffset`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`TZ`].

use crate::{
    prop::tz::TZ,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::utc_offset::IcalUtcOffset,
};

impl IcalPropLens for TZ {
    type Target<'v> = IcalUtcOffset<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
