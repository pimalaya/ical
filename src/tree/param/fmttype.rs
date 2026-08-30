//! # FMTTYPE parameter lens
//!
//! The `FMTTYPE` parameter lens: the media type (RFC 5545 3.2.8).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::IcalParamKind,
    tree::{
        codec::{escape::escape_param, mode::Escaper, unescape::unescape_param},
        leaf::IcalLeaf,
        param::{lens::IcalParamLens, node::IcalParamNode},
    },
};

/// The `FMTTYPE` parameter lens.
#[allow(non_camel_case_types)]
pub struct FMTTYPE;

impl IcalParamLens for FMTTYPE {
    const KIND: IcalParamKind = IcalParamKind::FmtType;

    type Target<'v> = Cow<'v, str>;

    fn decode<'v>(param: &'v IcalParamNode<'_>) -> Cow<'v, str> {
        param
            .values
            .first()
            .map(|value| unescape_param(value.get(), param.escaper))
            .unwrap_or_default()
    }

    fn encode(decoded: &Cow<'_, str>, escaper: Escaper) -> IcalParamNode<'static> {
        IcalParamNode {
            name: IcalLeaf::from(Self::KIND.to_string()),
            values: vec![IcalLeaf::from(escape_param(decoded, escaper).into_owned())],
            escaper,
        }
    }
}
