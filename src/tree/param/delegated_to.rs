//! # DELEGATED_TO parameter lens
//!
//! The `DELEGATED_TO` parameter lens: the delegatees (RFC 5545 3.2.5).

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::tree::leaf::IcalLeaf;
use crate::{
    param::IcalParamKind,
    tree::{codec::unescape::unescape, param::IcalParamLens, param::IcalParamNode},
};

/// The `DELEGATED_TO` parameter lens.
#[allow(non_camel_case_types)]
pub struct DELEGATED_TO;

impl IcalParamLens for DELEGATED_TO {
    const KIND: IcalParamKind = IcalParamKind::DelegatedTo;

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
