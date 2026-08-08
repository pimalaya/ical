//! # PERCENT_COMPLETE lens
//!
//! The `PERCENT_COMPLETE` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropCardinality, IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::integer::IcalInteger,
    version::IcalVersion,
};

/// The `PERCENT_COMPLETE` property lens.
#[allow(non_camel_case_types)]
pub struct PERCENT_COMPLETE;

impl IcalPropLens for PERCENT_COMPLETE {
    type Target<'v> = IcalInteger<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for PERCENT_COMPLETE {
    const KIND: IcalPropKind = IcalPropKind::PercentComplete;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Integer]
    }
}
