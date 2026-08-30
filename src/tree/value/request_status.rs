//! # REQUEST-STATUS value codec (RFC 5545 3.8.8.3)
//!
//! [`Codec`] for the structured `REQUEST-STATUS` value: a
//! `code;description;extra` triple, held as three `;`-separated components (the
//! third is optional and decodes to an empty value when absent).

use alloc::vec;

use crate::{
    tree::{
        codec::{Codec, encode::encode_component, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::request_status::IcalRequestStatus,
};

impl<'v> Codec<'v> for IcalRequestStatus<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalRequestStatus {
            code: node.decode_component(0),
            description: node.decode_component(1),
            extra: node.decode_component(2),
        }
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueNode::from_components(
            vec![
                encode_component(&[self.code.as_ref()], escaper),
                encode_component(&[self.description.as_ref()], escaper),
                encode_component(&[self.extra.as_ref()], escaper),
            ],
            escaper,
        )
    }
}
