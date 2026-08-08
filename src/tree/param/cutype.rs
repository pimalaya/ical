//! # CUTYPE parameter lens
//!
//! The `CUTYPE` parameter lens: the calendar user type (RFC 5545 3.2.3).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::IcalParamKind,
    tree::{
        codec::unescape::unescape,
        leaf::IcalLeaf,
        param::{IcalParamLens, IcalParamNode},
    },
};

/// The `CUTYPE` parameter lens.
#[allow(non_camel_case_types)]
pub struct CUTYPE;

impl IcalParamLens for CUTYPE {
    const KIND: IcalParamKind = IcalParamKind::CuType;

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
