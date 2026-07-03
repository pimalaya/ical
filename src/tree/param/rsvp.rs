//! # RSVP parameter lens
//!
//! The `RSVP` parameter lens: the RSVP expectation (RFC 5545 3.2.17).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::tree::leaf::IcalLeaf;
use crate::{
    param::IcalParamKind,
    tree::{codec::unescape::unescape, param::IcalParamLens, param::IcalParamNode},
};

/// The `RSVP` parameter lens.
#[allow(non_camel_case_types)]
pub struct RSVP;

impl IcalParamLens for RSVP {
    const KIND: IcalParamKind = IcalParamKind::Rsvp;

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
