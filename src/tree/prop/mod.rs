//! # Property lenses
//!
//! The property lens contract, the per-property spec, and one module per
//! iCalendar property.
//!
//! [`IcalPropLens`] ties a wire name to a decoded value type plus the `decode`
//! projection and an edit cursor; each property implements it on the marker in
//! its own module, the type-level key for
//! [`IcalCst::prop`](crate::tree::cst::IcalCst::prop). The per-property
//! contract is [`IcalPropSpec`], with the [`IcalPropCardinality`] multiplicity
//! axis; the name dispatch for whole-calendar decoding lives in
//! [`crate::tree::codec::decode`].

pub mod aalarm;
pub mod acknowledged;
pub mod action;
pub mod attach;
pub mod attendee;
pub mod busytype;
pub mod calendar_address;
pub mod calscale;
pub mod categories;
pub mod class;
pub mod color;
pub mod comment;
pub mod completed;
pub mod concept;
pub mod conference;
pub mod contact;
pub mod created;
pub mod dalarm;
pub mod description;
pub mod dtend;
pub mod dtstamp;
pub mod dtstart;
pub mod due;
pub mod duration;
pub mod exdate;
pub mod exrule;
pub mod freebusy;
pub mod geo;
pub mod image;
pub mod last_modified;
pub mod link;
pub mod location;
pub mod location_type;
pub mod malarm;
pub mod method;
pub mod name;
pub mod organizer;
pub mod palarm;
pub mod participant_type;
pub mod percent_complete;
pub mod priority;
pub mod prodid;
pub mod proximity;
pub mod rdate;
pub mod recurrence_id;
pub mod refid;
pub mod refresh_interval;
pub mod related_to;
pub mod repeat;
pub mod request_status;
pub mod resource_type;
pub mod resources;
pub mod rnum;
pub mod rrule;
pub mod sequence;
pub mod source;
pub mod status;
pub mod structured_data;
pub mod styled_description;
pub mod summary;
pub mod transp;
pub mod trigger;
pub mod tz;
pub mod tzid;
pub mod tzname;
pub mod tzoffsetfrom;
pub mod tzoffsetto;
pub mod tzurl;
pub mod uid;
pub mod url;

mod cardinality;
mod lens;
mod spec;

#[doc(inline)]
pub use cardinality::*;
#[doc(inline)]
pub use lens::*;
#[doc(inline)]
pub use spec::IcalPropSpec;

pub(crate) use spec::prop_spec;
