//! # Parameter lens contract
//!
//! [`IcalParamLens`] ties a wire name to a parameter's decoded shape (a single
//! value or a list); the per-name markers in the sibling modules implement it,
//! and it is the type-level key for
//! [`IcalLine::param`](crate::tree::line::IcalLine::param).

use crate::{
    param::IcalParamKind,
    tree::{codec::mode::Escaper, param::node::IcalParamNode},
};

/// A parameter identified by type, projected onto a decoded value and back.
///
/// The escaper is symmetric across the two directions, as on
/// [`Codec`](crate::tree::codec::Codec): decode reads it off the incoming
/// node, encode receives the target mode and applies it.
pub trait IcalParamLens {
    /// The parameter kind to look up by (its wire name comes through `Deref`).
    const KIND: IcalParamKind;

    /// The decoded value type, borrowing the syntax node for reads.
    type Target<'v>;

    /// Project the generic syntax parameter onto the decoded type.
    fn decode<'v>(param: &'v IcalParamNode<'_>) -> Self::Target<'v>;

    /// Encode a decoded value back into a generic syntax parameter (owned),
    /// for the given escaping mode.
    fn encode(decoded: &Self::Target<'_>, escaper: Escaper) -> IcalParamNode<'static>;
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::tree::{
        codec::mode::Escaper,
        param::{language::LANGUAGE, lens::IcalParamLens, member::MEMBER, node::IcalParamNode},
    };

    #[test]
    fn decodes_a_list_parameter_through_its_lens() {
        let node = IcalParamNode::parse("MEMBER=a,b");
        assert_eq!(
            MEMBER::decode(&node),
            vec![Cow::Borrowed("a"), Cow::Borrowed("b")],
        );
    }

    #[test]
    fn encodes_a_scalar_parameter_through_its_lens() {
        let node = LANGUAGE::encode(&Cow::Borrowed("en"), Escaper::Modern);
        assert_eq!(node.to_string(), "LANGUAGE=en");
    }
}
