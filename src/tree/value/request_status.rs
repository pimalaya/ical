//! # REQUEST-STATUS value codec (RFC 5545 3.8.8.3)
//!
//! [`Codec`] for the structured `REQUEST-STATUS` value: a
//! `code;description;extra` triple, held as three `;`-separated components (the
//! third is optional and decodes to an empty value when absent).

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
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
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalRequestStatus<'_> {
    /// Write the status as its components, each holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.code]);
        out.component(&[&*self.description]);

        // NOTE: The extra data is optional (RFC 5545 3.8.8.3), and an empty
        // one is no extra data, so it writes as none.
        if !self.extra.is_empty() {
            out.component(&[&*self.extra]);
        }
    }
}
