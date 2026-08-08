//! # CONTACT lens
//!
//! The `CONTACT` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::text::IcalText,
    version::IcalVersion,
};

/// The `CONTACT` property lens.
#[allow(non_camel_case_types)]
pub struct CONTACT;

impl IcalPropLens for CONTACT {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for CONTACT {
    const KIND: IcalPropKind = IcalPropKind::Contact;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
