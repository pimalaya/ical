//! # CONFERENCE lens
//!
//! The `CONFERENCE` property lens.

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

/// The `CONFERENCE` property lens.
#[allow(non_camel_case_types)]
pub struct CONFERENCE;

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

impl IcalPropSpec for CONFERENCE {
    const KIND: IcalPropKind = IcalPropKind::Conference;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri]
    }
}
