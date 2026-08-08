//! # ORGANIZER lens
//!
//! The `ORGANIZER` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropCardinality, IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::cal_address::IcalCalAddress,
    version::IcalVersion,
};

/// The `ORGANIZER` property lens.
#[allow(non_camel_case_types)]
pub struct ORGANIZER;

impl IcalPropLens for ORGANIZER {
    type Target<'v> = IcalCalAddress<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for ORGANIZER {
    const KIND: IcalPropKind = IcalPropKind::Organizer;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::CalAddress]
    }
}
