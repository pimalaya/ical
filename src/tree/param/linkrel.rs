//! # LINKREL parameter lens
//!
//! The `LINKREL` parameter lens: the relation type of a LINK (RFC 9253 6.1).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::IcalParamKind,
    tree::{
        codec::unescape::unescape,
        leaf::IcalLeaf,
        param::{lens::IcalParamLens, node::IcalParamNode},
    },
};

/// The `LINKREL` parameter lens.
#[allow(non_camel_case_types)]
pub struct LINKREL;

impl IcalParamLens for LINKREL {
    const KIND: IcalParamKind = IcalParamKind::LinkRel;

    type Target<'v> = Cow<'v, str>;

    fn decode<'v>(param: &'v IcalParamNode<'_>) -> Cow<'v, str> {
        param
            .values
            .first()
            .map(|value| unescape(value.get()))
            .unwrap_or_default()
    }

    fn encode(decoded: &Cow<'_, str>) -> IcalParamNode<'static> {
        IcalParamNode {
            name: IcalLeaf::from(Self::KIND.to_string()),
            values: vec![IcalLeaf::from(decoded.to_string())],
        }
    }
}
