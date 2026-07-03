//! # ALTREP parameter lens
//!
//! The `ALTREP` parameter lens: the alternate text representation URI (RFC 5545 3.2.1).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::tree::leaf::IcalLeaf;
use crate::{
    param::IcalParamKind,
    tree::{codec::unescape::unescape, param::IcalParamLens, param::IcalParamNode},
};

/// The `ALTREP` parameter lens.
#[allow(non_camel_case_types)]
pub struct ALTREP;

impl IcalParamLens for ALTREP {
    const KIND: IcalParamKind = IcalParamKind::AltRep;

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
