//! # DELEGATED_FROM parameter lens
//!
//! The `DELEGATED_FROM` parameter lens: the delegators (RFC 5545 3.2.4).

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::{
    param::IcalParamKind,
    tree::{
        codec::unescape::unescape,
        leaf::IcalLeaf,
        param::{IcalParamLens, IcalParamNode},
    },
};

/// The `DELEGATED_FROM` parameter lens.
#[allow(non_camel_case_types)]
pub struct DELEGATED_FROM;

impl IcalParamLens for DELEGATED_FROM {
    const KIND: IcalParamKind = IcalParamKind::DelegatedFrom;

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
