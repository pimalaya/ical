//! # RELATED_TO lens
//!
//! The `RELATED_TO` property lens.

use crate::{
    param::IcalParamKind,
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::text::IcalText,
    version::IcalVersion,
};

/// The `RELATED_TO` property lens.
#[allow(non_camel_case_types)]
pub struct RELATED_TO;

impl IcalPropLens for RELATED_TO {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for RELATED_TO {
    const KIND: IcalPropKind = IcalPropKind::RelatedTo;

    /// RFC 9253 6.2 adds `GAP` here, the lag or lead between the two related
    /// components, beside the RFC 5545 `RELTYPE`.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Value,
            IcalParamKind::RelType,
            IcalParamKind::Gap,
        ]
    }
}
