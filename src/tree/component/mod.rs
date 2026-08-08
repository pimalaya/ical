//! # Component lenses
//!
//! The component lens contract ([`IcalComponentLens`]), the per-component spec
//! ([`IcalComponentSpec`]) and one module per iCalendar component. The markers
//! are the type-level keys for typed subtree access
//! ([`IcalCst::component`](crate::tree::cst::IcalCst::component)); the spec
//! drives nesting and required-property checks in
//! [`crate::tree::ical::validate`].

pub mod available;
pub mod daylight;
pub mod participant;
pub mod standard;
pub mod valarm;
pub mod vavailability;
pub mod vcalendar;
pub mod vevent;
pub mod vfreebusy;
pub mod vjournal;
pub mod vlocation;
pub mod vresource;
pub mod vtimezone;
pub mod vtodo;

mod lens;
mod spec;

#[doc(inline)]
pub use lens::*;
#[doc(inline)]
pub use spec::IcalComponentSpec;

pub(crate) use spec::component_spec;
