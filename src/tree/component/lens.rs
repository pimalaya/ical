//! # Component lens contract
//!
//! [`IcalComponentLens`] identifies a component by type, the type-level key for
//! [`IcalCst::component`](crate::tree::cst::IcalCst::component). The wire name
//! comes from its [`IcalComponentSpec::KIND`] supertrait, so the two stay in
//! sync.

use crate::tree::component::spec::IcalComponentSpec;

/// A component identified by type. The wire name and nesting rules come from
/// its [`IcalComponentSpec`] supertrait.
pub trait IcalComponentLens: IcalComponentSpec {}
