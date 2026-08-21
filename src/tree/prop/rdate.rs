//! # RDATE lens
//!
//! The `RDATE` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::datetime::IcalDateTimeList,
    version::IcalVersion,
};

/// The `RDATE` property lens.
#[allow(non_camel_case_types)]
pub struct RDATE;

impl IcalPropLens for RDATE {
    type Target<'v> = IcalDateTimeList<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for RDATE {
    const KIND: IcalPropKind = IcalPropKind::RDate;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTimeList]
    }

    /// A list whatever its items are: a declared `DATE`, `DATE-TIME` or
    /// `PERIOD` describes each item, not the value as a whole, and every item
    /// is kept as raw text.
    fn value(_version: IcalVersion, _declared: Option<IcalValueKind>) -> IcalValueKind {
        IcalValueKind::DateTimeList
    }
}
