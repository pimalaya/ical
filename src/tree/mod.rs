//! # Syntax tree
//!
//! Everything syntactic: the byte-faithful representation of a calendar and the
//! bridges to and from the decoded model.
//!
//! The hub is [`cst::IcalCst`], a recursive tree of generic nodes
//! ([`line`](mod@line), [`param`], [`value`], [`leaf`]) that round-trips the
//! wire bytes exactly. On top of the generic tree sit the per-name lens
//! markers, each carrying the `IcalPropLens` / `IcalParamLens` /
//! `IcalComponentLens` contract (plus the per-property `IcalPropSpec` and
//! per-component `IcalComponentSpec`) defined in [`prop`] / [`param`] /
//! [`component`], the in-place edit cursor in [`value`], the [`codec`] that
//! projects between the tree and the decoded model (decode / encode plus the
//! value escaping), and the strict-out layer in [`ical`] (the spec-driven
//! builder and validation). Parsing is the only fallible step, so its
//! [`error`] type lives here too. This whole layer is gated behind the `parser`
//! feature, so the decoded model can be depended on without it.

pub mod codec;
pub mod component;
pub mod cst;
pub mod error;
pub mod ical;
pub mod leaf;
pub mod line;
pub mod param;
pub mod prop;
pub mod value;
