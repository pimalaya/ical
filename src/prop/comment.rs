//! # COMMENT
//!
//! The `COMMENT` property: a comment on the component (RFC 5545 3.8.1.4).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `COMMENT` property marker.
pub struct COMMENT;

impl IcalPropSpec for COMMENT {
    const KIND: IcalPropKind = IcalPropKind::Comment;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
