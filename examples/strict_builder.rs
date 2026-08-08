//! Build a calendar checked against the standard.
//!
//! Each property is built through the spec-driven builder, and the finished
//! calendar is validated as a whole (every component carries the properties its
//! spec requires) before it is serialized.
//!
//! Run with: `cargo run --example strict_builder`

use std::borrow::Cow;

use ical::{
    component::{IcalComponent, IcalComponentKind},
    ical::Ical,
    tree::{
        cst::IcalCst,
        ical::builder::IcalPropBuilder,
        prop::{dtstamp::DTSTAMP, prodid::PRODID, summary::SUMMARY, uid::UID},
    },
    value::{IcalValue, datetime::IcalDateTime, text::IcalText},
    version::IcalVersion,
};

fn main() {
    let version = IcalVersion::V2_0;

    let prodid = IcalPropBuilder::<PRODID>::new(version)
        .build(IcalValue::Text(IcalText(Cow::Borrowed("-//Example//EN"))))
        .unwrap();

    let uid = IcalPropBuilder::<UID>::new(version)
        .build(IcalValue::Text(IcalText(Cow::Borrowed("42@example.com"))))
        .unwrap();

    let dtstamp = IcalPropBuilder::<DTSTAMP>::new(version)
        .build(IcalValue::DateTime(IcalDateTime(Cow::Borrowed(
            "20260101T000000Z",
        ))))
        .unwrap();

    let summary = IcalPropBuilder::<SUMMARY>::new(version)
        .build(IcalValue::Text(IcalText(Cow::Borrowed("Lunch"))))
        .unwrap();

    let cal = Ical {
        version,
        props: vec![prodid],
        components: vec![IcalComponent {
            name: IcalComponentKind::VEvent.into(),
            props: vec![uid, dtstamp, summary],
            components: vec![],
        }],
    };

    let valid = cal.validate().expect("a conformant 2.0 calendar");

    print!("{}", IcalCst::from(valid));
}
