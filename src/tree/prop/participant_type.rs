//! # PARTICIPANT-TYPE lens
//!
//! The `PARTICIPANT-TYPE` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::text::IcalText,
    version::IcalVersion,
};

/// The `PARTICIPANT-TYPE` property lens.
#[allow(non_camel_case_types)]
pub struct PARTICIPANT_TYPE;

impl IcalPropLens for PARTICIPANT_TYPE {
    type Target<'v> = IcalText<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for PARTICIPANT_TYPE {
    const KIND: IcalPropKind = IcalPropKind::ParticipantType;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
