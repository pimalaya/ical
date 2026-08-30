//! # FREEBUSY lens
//!
//! Reading and editing the `FREEBUSY` property in place: it decodes as an
//! [`IcalPeriod`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`FREEBUSY`].

use crate::{
    prop::freebusy::FREEBUSY,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::period::IcalPeriod,
};

impl IcalPropLens for FREEBUSY {
    type Target<'v> = IcalPeriod<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
