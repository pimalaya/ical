//! # REQUEST-STATUS lens
//!
//! Reading and editing the `REQUEST-STATUS` property in place: it decodes as an
//! [`IcalRequestStatus`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`REQUEST_STATUS`].

use crate::{
    prop::request_status::REQUEST_STATUS,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::request_status::IcalRequestStatus,
};

impl IcalPropLens for REQUEST_STATUS {
    type Target<'v> = IcalRequestStatus<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
