//! # REFRESH-INTERVAL lens
//!
//! Reading and editing the `REFRESH-INTERVAL` property in place: it decodes as
//! an [`IcalDuration`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`REFRESH_INTERVAL`].

use crate::{
    prop::refresh_interval::REFRESH_INTERVAL,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::duration::IcalDuration,
};

impl IcalPropLens for REFRESH_INTERVAL {
    type Target<'v> = IcalDuration<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
