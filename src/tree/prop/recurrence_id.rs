//! # RECURRENCE-ID lens
//!
//! The `RECURRENCE-ID` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{cardinality::IcalPropCardinality, lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::datetime::IcalDateTime,
    version::IcalVersion,
};

/// The `RECURRENCE-ID` property lens.
#[allow(non_camel_case_types)]
pub struct RECURRENCE_ID;

impl IcalPropLens for RECURRENCE_ID {
    type Target<'v> = IcalDateTime<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for RECURRENCE_ID {
    const KIND: IcalPropKind = IcalPropKind::RecurrenceId;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
