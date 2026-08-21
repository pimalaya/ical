//! # LINK lens
//!
//! The `LINK` property lens.

use crate::{
    param::IcalParamKind,
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::uri::IcalUri,
    version::IcalVersion,
};

/// The `LINK` property lens.
#[allow(non_camel_case_types)]
pub struct LINK;

impl IcalPropLens for LINK {
    type Target<'v> = IcalUri<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for LINK {
    const KIND: IcalPropKind = IcalPropKind::Link;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri, IcalValueKind::Text]
    }

    /// RFC 9253 8.1: a link states what it links to with `LINKREL`.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Value,
            IcalParamKind::LinkRel,
            IcalParamKind::Label,
            IcalParamKind::FmtType,
        ]
    }
}
