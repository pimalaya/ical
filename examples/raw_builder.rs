//! Assemble a calendar by hand and export it, with no checks.
//!
//! This is the escape hatch: you place whatever properties and components you
//! like, including extensions the library has never heard of, and write them
//! out directly. Correctness is your responsibility.
//!
//! Run with: `cargo run --example raw_builder`

use std::borrow::Cow;

use ical::{
    component::{IcalComponent, IcalComponentKind},
    ical::Ical,
    prop::IcalProp,
    value::{IcalValue, datetime::IcalDateTime, text::IcalText},
    version::IcalVersion,
};

fn main() {
    let cal = Ical {
        version: IcalVersion::V2_0,
        props: vec![IcalProp {
            name: "PRODID".into(),
            params: vec![],
            value: IcalValue::Text(IcalText(Cow::Borrowed("-//Example//EN"))),
        }],
        components: vec![IcalComponent {
            name: IcalComponentKind::VEvent.into(),
            props: vec![
                IcalProp {
                    name: "UID".into(),
                    params: vec![],
                    value: IcalValue::Text(IcalText(Cow::Borrowed("42@example.com"))),
                },
                IcalProp {
                    name: "DTSTAMP".into(),
                    params: vec![],
                    value: IcalValue::DateTime(IcalDateTime(Cow::Borrowed("20260101T000000Z"))),
                },
                // An unvalidated extension property; it round-trips untouched.
                IcalProp {
                    name: "X-CUSTOM".into(),
                    params: vec![],
                    value: IcalValue::Text(IcalText(Cow::Borrowed("anything"))),
                },
            ],
            components: vec![],
        }],
    };

    print!("{cal}");
}
