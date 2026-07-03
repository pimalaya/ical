//! # CREATED lens
//!
//! The `CREATED` property lens.

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

/// The `CREATED` property lens.
#[allow(non_camel_case_types)]
pub struct CREATED;

impl IcalPropLens for CREATED {
    type Target<'v> = IcalDateTime<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for CREATED {
    const KIND: IcalPropKind = IcalPropKind::Created;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
