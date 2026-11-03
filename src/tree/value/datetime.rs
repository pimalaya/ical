//! # Date and time value codecs (RFC 5545 3.3.4, 3.3.5, 3.3.12)
//!
//! [`Codec`] for the DATE, DATE-TIME and TIME values, each kept as raw text.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::datetime::{IcalDate, IcalDateTime, IcalDateTimeList, IcalTime},
};

impl<'v> Codec<'v> for IcalDate<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalDate(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalDate<'_> {
    /// Write the date as one component holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}

impl<'v> Codec<'v> for IcalDateTimeList<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalDateTimeList(node.decode_list())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalDateTimeList<'_> {
    /// Write the list as one component of its values.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&self.0);
    }
}

impl<'v> Codec<'v> for IcalDateTime<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalDateTime(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalDateTime<'_> {
    /// Write the date-time as one component holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}

impl<'v> Codec<'v> for IcalTime<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalTime(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalTime<'_> {
    /// Write the time as one component holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}
