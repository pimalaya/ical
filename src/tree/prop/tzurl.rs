//! # TZURL lens
//!
//! The `TZURL` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{cardinality::IcalPropCardinality, lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::uri::IcalUri,
    version::IcalVersion,
};

/// The `TZURL` property lens.
#[allow(non_camel_case_types)]
pub struct TZURL;

impl IcalPropLens for TZURL {
    type Target<'v> = IcalUri<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for TZURL {
    const KIND: IcalPropKind = IcalPropKind::TzUrl;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri]
    }
}
