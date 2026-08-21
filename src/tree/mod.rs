//! # Syntax tree
//!
//! Everything syntactic: the byte-faithful representation of a calendar and the
//! bridges to and from the decoded model.
//!
//! The hub is [`cst::IcalCst`], a recursive tree of generic nodes
//! ([`line`](mod@line), [`param`], [`value`], [`leaf`], with each line's wire
//! layout on [`wire`]) that round-trips the wire bytes exactly. On top of it sit
//! the per-name lens markers in [`prop`] / [`param`] / [`component`], each
//! carrying its lens contract (and, for a property or a component, its spec),
//! the in-place edit cursor in [`value`], the [`codec`] projecting between tree
//! and decoded model, the strict-out layer in [`ical`] (the spec-driven builder
//! and validation), and the three-way [`merge`](mod@merge) reconciling two
//! divergent edits against their common base. Parsing is the only fallible step,
//! so its [`error`] type lives here too. This whole layer is gated behind the
//! `parser` feature, so the decoded model can be depended on without it.
pub mod codec;
pub mod component;
pub mod cst;
pub mod error;
pub mod ical;
pub mod leaf;
pub mod line;
pub mod merge;
pub mod param;
pub mod prop;
pub mod value;
pub mod wire;
