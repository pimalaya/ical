//! # SCHEDULE-FORCE-SEND parameter lens
//!
//! The `SCHEDULE-FORCE-SEND` parameter lens: a request to resend a scheduling
//! message (RFC 6638 7.2).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::IcalParamKind,
    tree::{
        codec::{escape::escape_param, mode::Escaper, unescape::unescape_param},
        leaf::IcalLeaf,
        param::{lens::IcalParamLens, node::IcalParamNode},
    },
};

/// The `SCHEDULE-FORCE-SEND` parameter lens.
#[allow(non_camel_case_types)]
pub struct SCHEDULE_FORCE_SEND;

impl IcalParamLens for SCHEDULE_FORCE_SEND {
    const KIND: IcalParamKind = IcalParamKind::ScheduleForceSend;

    type Target<'v> = Cow<'v, str>;

    fn decode<'v>(param: &'v IcalParamNode<'_>) -> Cow<'v, str> {
        param
            .values
            .first()
            .map(|value| unescape_param(value.get(), param.escaper))
            .unwrap_or_default()
    }

    fn encode(decoded: &Cow<'_, str>, escaper: Escaper) -> IcalParamNode<'static> {
        IcalParamNode {
            name: IcalLeaf::from(Self::KIND.to_string()),
            values: vec![IcalLeaf::from(escape_param(decoded, escaper).into_owned())],
            escaper,
        }
    }
}
