//! # RESOURCES lens
//!
//! The `RESOURCES` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::text::IcalTextList,
    version::IcalVersion,
};

/// The `RESOURCES` property lens.
#[allow(non_camel_case_types)]
pub struct RESOURCES;

impl IcalPropLens for RESOURCES {
    type Target<'v> = IcalTextList<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for RESOURCES {
    const KIND: IcalPropKind = IcalPropKind::Resources;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::TextList]
    }

    /// A list whatever `VALUE` declares: `TEXT` describes each item, not the
    /// value as a whole.
    fn value(_version: IcalVersion, _declared: Option<IcalValueKind>) -> IcalValueKind {
        IcalValueKind::TextList
    }
}
