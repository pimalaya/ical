//! # CN parameter lens
//!
//! The `CN` parameter lens: the common name (RFC 5545 3.2.2).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::IcalParamKind,
    tree::{
        codec::{escape::escape_param, mode::Escaper, unescape::unescape_param},
        leaf::IcalLeaf,
        param::{lens::IcalParamLens, node::IcalParamNode},
    },
};

/// The `CN` parameter lens.
#[allow(non_camel_case_types)]
pub struct CN;

impl IcalParamLens for CN {
    const KIND: IcalParamKind = IcalParamKind::Cn;

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
