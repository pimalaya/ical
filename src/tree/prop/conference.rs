//! # CONFERENCE lens
//!
//! Reading and editing the `CONFERENCE` property in place: it decodes as an
//! [`IcalUri`] and edits through the generic [`IcalValueCursor`].
//!
//! Its RFC contract sits on the marker, [`CONFERENCE`].

use crate::{
    prop::conference::CONFERENCE,
    tree::{line::IcalLine, prop::lens::IcalPropLens, value::cursor::IcalValueCursor},
    value::uri::IcalUri,
};

impl IcalPropLens for CONFERENCE {
    type Target<'v> = IcalUri<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}
