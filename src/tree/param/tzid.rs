//! # TZID parameter lens
//!
//! The `TZID` parameter lens: the time-zone reference (RFC 5545 3.2.19).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::tree::leaf::IcalLeaf;
use crate::{
    param::IcalParamKind,
    tree::{codec::unescape::unescape, param::IcalParamLens, param::IcalParamNode},
};

/// The `TZID` parameter lens.
#[allow(non_camel_case_types)]
pub struct TZID;

impl IcalParamLens for TZID {
    const KIND: IcalParamKind = IcalParamKind::TzId;

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
