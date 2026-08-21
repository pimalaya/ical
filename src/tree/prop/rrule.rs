//! # RRULE lens
//!
//! The `RRULE` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::recur::IcalRecur,
    version::IcalVersion,
};

/// The `RRULE` property lens.
#[allow(non_camel_case_types)]
pub struct RRULE;

impl IcalPropLens for RRULE {
    type Target<'v> = IcalRecur<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for RRULE {
    const KIND: IcalPropKind = IcalPropKind::RRule;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Recur]
    }
}
