//! # FEATURE parameter lens
//!
//! The `FEATURE` parameter lens: the conference feature set (RFC 7986 6.3).

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::{
    param::IcalParamKind,
    tree::{
        codec::unescape::unescape,
        leaf::IcalLeaf,
        param::{IcalParamLens, IcalParamNode},
    },
};

/// The `FEATURE` parameter lens.
#[allow(non_camel_case_types)]
pub struct FEATURE;

impl IcalParamLens for FEATURE {
    const KIND: IcalParamKind = IcalParamKind::Feature;

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
