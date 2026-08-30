//! Lens isolation: editing one property through its lens leaves every other
//! byte of the calendar untouched, including nested components.

#![cfg(feature = "parser")]

use ical::{
    component::{valarm::VALARM, vevent::VEVENT},
    prop::{description::DESCRIPTION, summary::SUMMARY},
    tree::cst::IcalCst,
};

const CAL: &str = concat!(
    "BEGIN:VCALENDAR\r\n",
    "VERSION:2.0\r\n",
    "PRODID:-//Example//EN\r\n",
    "BEGIN:VEVENT\r\n",
    "UID:1@example.com\r\n",
    "DTSTAMP:20260101T000000Z\r\n",
    "DTSTART:20260102T120000Z\r\n",
    "SUMMARY:Lunch\r\n",
    "BEGIN:VALARM\r\n",
    "ACTION:DISPLAY\r\n",
    "DESCRIPTION:Reminder\r\n",
    "TRIGGER:-PT15M\r\n",
    "END:VALARM\r\n",
    "END:VEVENT\r\n",
    "END:VCALENDAR\r\n",
);

#[test]
fn editing_a_nested_event_property_changes_only_that_line() {
    let mut cal = IcalCst::parse(CAL).unwrap();
    cal.component_mut::<VEVENT>()
        .unwrap()
        .prop_mut::<SUMMARY>()
        .unwrap()
        .set_text("Dinner");

    assert_eq!(
        cal.to_string(),
        CAL.replace("SUMMARY:Lunch", "SUMMARY:Dinner")
    );
}

#[test]
fn editing_a_deeply_nested_alarm_property_changes_only_that_line() {
    let mut cal = IcalCst::parse(CAL).unwrap();
    cal.component_mut::<VEVENT>()
        .unwrap()
        .component_mut::<VALARM>()
        .unwrap()
        .prop_mut::<DESCRIPTION>()
        .unwrap()
        .set_text("Wake up");

    assert_eq!(
        cal.to_string(),
        CAL.replace("DESCRIPTION:Reminder", "DESCRIPTION:Wake up"),
    );
}

#[test]
fn reads_a_nested_value_through_the_lens() {
    let cal = IcalCst::parse(CAL).unwrap();
    let event = cal.component::<VEVENT>().unwrap();
    assert_eq!(&*event.prop::<SUMMARY>().unwrap().0, "Lunch");

    let alarm = event.component::<VALARM>().unwrap();
    assert_eq!(&*alarm.prop::<DESCRIPTION>().unwrap().0, "Reminder");
}
