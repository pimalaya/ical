//! Reconcile two divergent edits of the same calendar against their common
//! base.
//!
//! This is what a synchronisation engine needs: the phone and the server both
//! edited the calendar they last agreed on, and the merge says what each side
//! did, where they collided, and hands back one calendar carrying both sets of
//! changes.
//!
//! Run with: `cargo run --example three_way_merge`

use ical::tree::{cst::IcalCst, merge::IcalMerge};

fn main() {
    let base = IcalCst::parse(concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:sync@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260105T090000Z\r\n",
        "SUMMARY:Weekly sync\r\n",
        "LOCATION:Room A\r\n",
        "ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:ada@example.com\r\n",
        "CATEGORIES:work\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    ))
    .unwrap();

    // The phone moved the event to another room, invited a second attendee and
    // added a category.
    let left = IcalCst::parse(concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:sync@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260105T090000Z\r\n",
        "SUMMARY:Weekly sync\r\n",
        "LOCATION:Room B\r\n",
        "ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:ada@example.com\r\n",
        "ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:grace@example.com\r\n",
        "CATEGORIES:work,weekly\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    ))
    .unwrap();

    // The server retitled it, moved it to a third room, and recorded that Ada
    // accepted.
    let right = IcalCst::parse(concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:sync@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260105T090000Z\r\n",
        "SUMMARY:Weekly sync (renamed)\r\n",
        "LOCATION:Room C\r\n",
        "ATTENDEE;PARTSTAT=ACCEPTED:mailto:ada@example.com\r\n",
        "CATEGORIES:work\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    ))
    .unwrap();

    let report = IcalMerge {
        base: &base,
        left: &left,
        right: &right,
    }
    .merge();

    println!("left did:");
    for action in &report.left {
        println!("  {action:?}");
    }

    println!("\nright did:");
    for action in &report.right {
        println!("  {action:?}");
    }

    // Both sides moved the event, to different rooms. The left side wins in the
    // merged calendar, but the collision is reported, so a caller free to
    // resolve it otherwise can.
    println!("\nconflicts:");
    for conflict in &report.conflicts {
        println!("  right wanted {:?}", conflict.right);
        println!("  because      {:?}", conflict.reason);
    }

    // The merged calendar is the left calendar's bytes with the right side's
    // non-conflicting changes replayed onto them: the rename and the RSVP land,
    // the room stays the left side's, and the attendee the phone invited is
    // still there.
    print!("\n{}", report.merged);
}
