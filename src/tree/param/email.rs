//! # EMAIL parameter lens
//!
//! The `EMAIL` parameter lens: the email address (RFC 7986 6.2).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::IcalParamKind,
    tree::{
        codec::unescape::unescape,
        leaf::IcalLeaf,
        param::{IcalParamLens, IcalParamNode},
    },
};

/// The `EMAIL` parameter lens.
#[allow(non_camel_case_types)]
pub struct EMAIL;

impl IcalParamLens for EMAIL {
    const KIND: IcalParamKind = IcalParamKind::Email;

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
