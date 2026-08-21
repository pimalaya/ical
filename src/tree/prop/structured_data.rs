//! # STRUCTURED-DATA lens
//!
//! The `STRUCTURED-DATA` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::text::IcalText,
    version::IcalVersion,
};

/// The `STRUCTURED-DATA` property lens.
#[allow(non_camel_case_types)]
pub struct STRUCTURED_DATA;

impl IcalPropLens for STRUCTURED_DATA {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for STRUCTURED_DATA {
    const KIND: IcalPropKind = IcalPropKind::StructuredData;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
