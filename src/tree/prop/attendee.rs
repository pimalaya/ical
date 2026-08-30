//! # ATTENDEE lens
//!
//! Reading and editing the `ATTENDEE` property in place: it decodes as an
//! [`IcalCalAddress`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`ATTENDEE`].

use crate::{
    prop::attendee::ATTENDEE,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::cal_address::IcalCalAddress,
};

impl IcalPropLens for ATTENDEE {
    type Target<'v> = IcalCalAddress<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
