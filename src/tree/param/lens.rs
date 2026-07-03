//! # Parameter lens contract
//!
//! [`IcalParamLens`] ties a wire name to a parameter's decoded shape (a single
//! value or a list); the per-name markers in the sibling modules implement it,
//! and it is the type-level key for
//! [`IcalLine::param`](crate::tree::line::IcalLine::param).

use crate::{param::IcalParamKind, tree::param::IcalParamNode};

/// A parameter identified by type, projecting a generic syntax parameter onto a
/// decoded value type and back.
pub trait IcalParamLens {
    /// The parameter kind to look up by (its wire name comes through `Deref`).
    const KIND: IcalParamKind;

    /// The decoded value type, borrowing the syntax node for reads.
    type Target<'v>;

    /// Project the generic syntax parameter onto the decoded type.
    fn decode<'v>(param: &'v IcalParamNode<'_>) -> Self::Target<'v>;

    /// Encode a decoded value back into a generic syntax parameter (owned).
    fn encode(decoded: &Self::Target<'_>) -> IcalParamNode<'static>;
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::tree::param::{IcalParamLens, IcalParamNode, language::LANGUAGE, member::MEMBER};

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
        let node = LANGUAGE::encode(&Cow::Borrowed("en"));
        assert_eq!(node.to_string(), "LANGUAGE=en");
    }
}
