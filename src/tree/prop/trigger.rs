//! # TRIGGER lens
//!
//! The `TRIGGER` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::duration::IcalDuration,
    version::IcalVersion,
};

/// The `TRIGGER` property lens.
#[allow(non_camel_case_types)]
pub struct TRIGGER;

impl IcalPropLens for TRIGGER {
    type Target<'v> = IcalDuration<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for TRIGGER {
    const KIND: IcalPropKind = IcalPropKind::Trigger;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Duration]
    }
}
