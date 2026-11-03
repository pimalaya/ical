//! # Parameter node
//!
//! The raw, byte-faithful parameter on the syntax side.
//!
//! [`IcalParamNode`] is the syntactic peer of the decoded
//! [`IcalParam`](crate::param::IcalParam): a name leaf and its raw value
//! leaves, parsed from and serialized back to the wire verbatim. The per-name
//! parameter lenses that give it meaning live alongside in
//! [`crate::tree::param`].

use core::{fmt, mem};

use alloc::{borrow::Cow, vec::Vec};

use crate::tree::{
    codec::mode::Escaper,
    leaf::{IcalLeaf, put},
};

/// One raw parameter: a name and its `,`-separated raw values (empty when the
/// parameter has no `=` list). The syntactic peer of the decoded
/// [`IcalParam`](crate::param::IcalParam).
#[derive(Clone, Debug)]
pub struct IcalParamNode<'a> {
    /// The parameter name leaf.
    pub name: IcalLeaf<'a>,
    /// The raw value leaves.
    pub values: Vec<IcalLeaf<'a>>,
    /// Which version's parameter encoding rules the values are written in.
    /// Stamped by the parser once `VERSION` is known, as on a value node.
    pub escaper: Escaper,
}

impl<'a> IcalParamNode<'a> {
    /// Parse one `name=value,value` parameter (commas outside quotes split).
    pub fn parse(param: &'a str) -> Self {
        match param.split_once('=') {
            Some((name, values)) => Self {
                name: IcalLeaf::from(name),
                values: split_param_values(values),
                escaper: Escaper::default(),
            },
            None => Self {
                name: IcalLeaf::from(param),
                values: Vec::new(),
                escaper: Escaper::default(),
            },
        }
    }

    /// Take one parameter off the front of `rest`, the head bytes after a `;`,
    /// up to the next `;` outside double quotes, onto `params`, and return
    /// what follows that `;`, if one ends it.
    ///
    /// One walk does what [`parse`](Self::parse) does once the head splitter
    /// found the `;`: the first `=` ends the name, and a `,` outside quotes
    /// counted from that `=` on splits two values.
    ///
    /// The node is pushed first and its values pushed where it lands: a node
    /// built aside and moved in waits on the stores of its value list.
    pub(crate) fn take_into(rest: &'a str, params: &mut Vec<Self>) -> Option<&'a str> {
        params.push(const { Self::empty() });
        let at = params.len() - 1;
        let node = &mut params[at];

        let mut quoted = false;
        let mut values_quoted = false;
        let mut eq = None;
        let mut start = 0;
        let mut end = rest.len();
        let mut tail = None;

        for (i, &byte) in rest.as_bytes().iter().enumerate() {
            match byte {
                b'"' => {
                    quoted = !quoted;
                    values_quoted = !values_quoted;
                }
                b';' if !quoted => {
                    end = i;
                    tail = Some(&rest[i + 1..]);
                    break;
                }
                b'=' if eq.is_none() => {
                    eq = Some(i);
                    start = i + 1;
                    values_quoted = false;
                }
                b',' if eq.is_some() && !values_quoted => {
                    node.values.push(IcalLeaf::from(&rest[start..i]));
                    start = i + 1;
                }
                _ => {}
            }
        }

        let name = match eq {
            Some(eq) => {
                node.values.push(IcalLeaf::from(&rest[start..end]));
                &rest[..eq]
            }
            None => &rest[..end],
        };

        // NOTE: The placeholder's name holds no allocation.
        mem::forget(mem::replace(&mut node.name, IcalLeaf::from(name)));

        tail
    }

    /// A placeholder node for [`take_into`](Self::take_into) to fill in place.
    const fn empty() -> Self {
        Self {
            name: IcalLeaf(Cow::Borrowed("")),
            values: Vec::new(),
            escaper: Escaper::Modern,
        }
    }

    /// Convert into an owned parameter node (`'static`).
    pub(crate) fn into_static(self) -> IcalParamNode<'static> {
        IcalParamNode {
            name: self.name.into_static(),
            values: self.values.into_iter().map(IcalLeaf::into_static).collect(),
            escaper: self.escaper,
        }
    }

    /// Serialize the parameter (`name` or `name=value,value`) into `out`,
    /// without the intermediate `String` a `Display`-based path would allocate.
    pub(crate) fn write_bytes(&self, out: &mut Vec<u8>) {
        put(out, self.name.get().as_bytes());

        if let Some((first, rest)) = self.values.split_first() {
            out.push(b'=');
            put(out, first.get().as_bytes());
            for value in rest {
                out.push(b',');
                put(out, value.get().as_bytes());
            }
        }
    }
}

impl fmt::Display for IcalParamNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.get())?;

        if let Some((first, rest)) = self.values.split_first() {
            write!(f, "={}", first.get())?;

            for value in rest {
                write!(f, ",{}", value.get())?;
            }
        }

        Ok(())
    }
}

/// Split a parameter value list on commas outside double quotes.
fn split_param_values(values: &str) -> Vec<IcalLeaf<'_>> {
    let bytes = values.as_bytes();
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut quoted = false;

    for (i, &byte) in bytes.iter().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b',' if !quoted => {
                pieces.push(IcalLeaf::from(&values[start..i]));
                start = i + 1;
            }
            _ => {}
        }
    }

    pieces.push(IcalLeaf::from(&values[start..]));
    pieces
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::param::node::IcalParamNode;

    #[test]
    fn parses_quoted_values_then_round_trips() {
        let node = IcalParamNode::parse(r#"TYPE=work,"a,b""#);
        assert_eq!(node.values.len(), 2);
        assert_eq!(node.to_string(), r#"TYPE=work,"a,b""#);
    }
}
