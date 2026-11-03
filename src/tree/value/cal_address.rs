//! # Calendar user address value codec (RFC 5545 3.3.3)
//!
//! [`Codec`] for the calendar user address value. A calendar user address is a
//! URI, so it reads and writes as one: the whole value is read, its `;` and `,`
//! kept as part of the address, and written back exactly as it is held, with
//! no text escaping (RFC 5545 3.3.13).

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::cal_address::IcalCalAddress,
};

impl<'v> Codec<'v> for IcalCalAddress<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalCalAddress(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalCalAddress<'_> {
    /// Write the address as the URI it is.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.uri(&self.0);
    }
}
