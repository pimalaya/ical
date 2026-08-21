//! # TZID lens
//!
//! The `TZID` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{cardinality::IcalPropCardinality, lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::text::IcalText,
    version::IcalVersion,
};

/// The `TZID` property lens.
#[allow(non_camel_case_types)]
pub struct TZID;

impl IcalPropLens for TZID {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for TZID {
    const KIND: IcalPropKind = IcalPropKind::TzId;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
