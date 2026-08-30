//! # TRIGGER lens
//!
//! Reading and editing the `TRIGGER` property in place: it decodes as an
//! [`IcalDuration`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`TRIGGER`].

use crate::{
    prop::trigger::TRIGGER,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::duration::IcalDuration,
};

impl IcalPropLens for TRIGGER {
    type Target<'v> = IcalDuration<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
