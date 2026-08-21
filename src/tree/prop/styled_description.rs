//! # STYLED-DESCRIPTION lens
//!
//! The `STYLED-DESCRIPTION` property lens.

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

/// The `STYLED-DESCRIPTION` property lens.
#[allow(non_camel_case_types)]
pub struct STYLED_DESCRIPTION;

impl IcalPropLens for STYLED_DESCRIPTION {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for STYLED_DESCRIPTION {
    const KIND: IcalPropKind = IcalPropKind::StyledDescription;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
