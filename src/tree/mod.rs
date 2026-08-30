//! # Syntax tree
//!
//! Everything syntactic: the byte-faithful representation of a calendar and the
//! bridges to and from the decoded model.
//!
//! The hub is [`cst::IcalCst`], a recursive tree of generic nodes
//! ([`line`](mod@line), [`param`], [`value`], [`leaf`], with each line's wire
//! layout on [`wire`]) that round-trips the wire bytes exactly.
//!
//! On top of it sit the read-and-edit lenses: one per property in [`prop`],
//! one per parameter in [`param`], and the in-place edit cursor in [`value`].
//! A property lens is implemented on the marker its property defines in
//! [`crate::prop`], so the RFC contract and the syntax projection meet on one
//! type without the contract needing a parser.
//!
//! Around them, [`codec`] projects between tree and decoded model, and the
//! three-way [`merge`](mod@merge) reconciles two divergent edits against
//! their common base.
//!
//! Parsing is the only fallible step, so its [`error`] type lives here too.
//! This whole layer is gated behind the `parser` feature, so the decoded
//! model can be depended on without it.

pub mod codec;
pub mod cst;
pub mod error;
pub mod leaf;
pub mod line;
pub mod merge;
pub mod param;
pub mod prop;
pub mod value;
pub mod wire;
