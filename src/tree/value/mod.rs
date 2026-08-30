//! # Values (syntax side)
//!
//! The raw value of a content line, the cursor that edits it in place, and the
//! per-value-type [`Codec`](crate::tree::codec::Codec) impls, one module each,
//! mirroring the model's `value/`.
//!
//! [`IcalValueNode`](node::IcalValueNode) is the generic, byte-faithful
//! value: `;`-separated components of `,`-separated leaves.
//!
//! [`IcalValueCursor`](cursor::IcalValueCursor) reads and writes it through
//! the codec, escaping on write and preserving every component it does not
//! touch. What the components *mean* is the lens's business (see
//! [`crate::tree::prop`]).

pub mod cursor;
pub mod node;

mod binary;
mod boolean;
mod cal_address;
mod datetime;
mod duration;
mod float;
mod geo;
mod integer;
mod period;
mod recur;
mod request_status;
mod text;
mod unknown;
mod uri;
mod utc_offset;
