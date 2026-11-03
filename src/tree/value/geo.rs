//! # GEO value codec (RFC 5545 3.8.1.6)
//!
//! [`Codec`] for the structured `GEO` value: a `latitude;longitude` pair of
//! FLOATs, held as two `;`-separated components.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::geo::IcalGeo,
};

impl<'v> Codec<'v> for IcalGeo<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalGeo {
            latitude: node.decode_component(0),
            longitude: node.decode_component(1),
        }
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalGeo<'_> {
    /// Write the pair as two components, each holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.latitude]);
        out.component(&[&*self.longitude]);
    }
}
