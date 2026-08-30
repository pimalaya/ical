//! # MEMBER parameter lens
//!
//! The `MEMBER` parameter lens: the group memberships (RFC 5545 3.2.11).

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::{
    param::IcalParamKind,
    tree::{
        codec::{escape::escape_param, mode::Escaper, unescape::unescape_param},
        leaf::IcalLeaf,
        param::{lens::IcalParamLens, node::IcalParamNode},
    },
};

/// The `MEMBER` parameter lens.
#[allow(non_camel_case_types)]
pub struct MEMBER;

impl IcalParamLens for MEMBER {
    const KIND: IcalParamKind = IcalParamKind::Member;

    type Target<'v> = Vec<Cow<'v, str>>;

    fn decode<'v>(param: &'v IcalParamNode<'_>) -> Vec<Cow<'v, str>> {
        param
            .values
            .iter()
            .map(|value| unescape_param(value.get(), param.escaper))
            .collect()
    }

    #[allow(clippy::ptr_arg)]
    fn encode(decoded: &Vec<Cow<'_, str>>, escaper: Escaper) -> IcalParamNode<'static> {
        IcalParamNode {
            name: IcalLeaf::from(Self::KIND.to_string()),
            values: decoded
                .iter()
                .map(|value| IcalLeaf::from(escape_param(value, escaper).into_owned()))
                .collect(),
            escaper,
        }
    }
}
