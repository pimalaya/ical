//! # MEMBER parameter lens
//!
//! The `MEMBER` parameter lens: the group memberships (RFC 5545 3.2.11).

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::{
    param::IcalParamKind,
    tree::{
        codec::unescape::unescape,
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
            .map(|value| unescape(value.get()))
            .collect()
    }

    #[allow(clippy::ptr_arg)]
    fn encode(decoded: &Vec<Cow<'_, str>>) -> IcalParamNode<'static> {
        IcalParamNode {
            name: IcalLeaf::from(Self::KIND.to_string()),
            values: decoded
                .iter()
                .map(|value| IcalLeaf::from(value.to_string()))
                .collect(),
        }
    }
}
