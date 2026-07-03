//! # EXRULE lens
//!
//! The `EXRULE` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::recur::IcalRecur,
    version::IcalVersion,
};

/// The `EXRULE` property lens.
#[allow(non_camel_case_types)]
pub struct EXRULE;

impl IcalPropLens for EXRULE {
    type Target<'v> = IcalRecur<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for EXRULE {
    const KIND: IcalPropKind = IcalPropKind::ExRule;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Recur]
    }
}
