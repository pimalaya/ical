//! # DURATION lens
//!
//! The `DURATION` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropCardinality, IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::duration::IcalDuration,
    version::IcalVersion,
};

/// The `DURATION` property lens.
#[allow(non_camel_case_types)]
pub struct DURATION;

impl IcalPropLens for DURATION {
    type Target<'v> = IcalDuration<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for DURATION {
    const KIND: IcalPropKind = IcalPropKind::Duration;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Duration]
    }
}
