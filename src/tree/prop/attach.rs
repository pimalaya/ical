//! # ATTACH lens
//!
//! The `ATTACH` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::uri::IcalUri,
    version::IcalVersion,
};

/// The `ATTACH` property lens.
#[allow(non_camel_case_types)]
pub struct ATTACH;

impl IcalPropLens for ATTACH {
    type Target<'v> = IcalUri<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for ATTACH {
    const KIND: IcalPropKind = IcalPropKind::Attach;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri]
    }
}
