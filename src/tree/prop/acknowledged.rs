//! # ACKNOWLEDGED lens
//!
//! The `ACKNOWLEDGED` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::datetime::IcalDateTime,
    version::IcalVersion,
};

/// The `ACKNOWLEDGED` property lens.
#[allow(non_camel_case_types)]
pub struct ACKNOWLEDGED;

impl IcalPropLens for ACKNOWLEDGED {
    type Target<'v> = IcalDateTime<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for ACKNOWLEDGED {
    const KIND: IcalPropKind = IcalPropKind::Acknowledged;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
