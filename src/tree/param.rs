//! # Parameters (syntax side)
//!
//! The per-parameter lens contract ([`IcalParamLens`](lens::IcalParamLens))
//! and one module per iCalendar parameter, tying a wire name to its decoded
//! shape.
//!
//! The markers are the type-level keys for
//! [`IcalLine::param`](crate::tree::line::IcalLine::param). The raw
//! [`IcalParamNode`](node::IcalParamNode) they read and write is defined
//! alongside.

pub mod altrep;
pub mod charset;
pub mod cn;
pub mod cutype;
pub mod delegated_from;
pub mod delegated_to;
pub mod derived;
pub mod dir;
pub mod display;
pub mod email;
pub mod encoding;
pub mod fbtype;
pub mod feature;
pub mod fmttype;
pub mod gap;
pub mod label;
pub mod language;
pub mod linkrel;
pub mod member;
pub mod order;
pub mod partstat;
pub mod range;
pub mod related;
pub mod reltype;
pub mod role;
pub mod rsvp;
pub mod schedule_agent;
pub mod schedule_force_send;
pub mod schedule_status;
pub mod schema;
pub mod sent_by;
pub mod tzid;
pub mod value;

pub mod lens;
pub mod node;

use crate::param::IcalParamKind;

/// The default parameters a property may carry, used by the spec for the
/// uniform majority. Per-property sets refine this where a property allows more
/// or fewer.
pub(crate) const COMMON_PARAMS: &[IcalParamKind] = &[
    IcalParamKind::Value,
    IcalParamKind::Language,
    IcalParamKind::AltRep,
];
