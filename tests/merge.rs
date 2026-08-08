//! The three-way merge.
//!
//! Every case states a base and two edits of it, and asserts three things: the
//! merged bytes, what each side is reported to have done, and which of the
//! right side's actions did not simply apply. The bytes matter as much as the
//! actions: a merge that reformats a line nobody touched has lost the property
//! the whole syntax tree exists to keep.

#![cfg(feature = "parser")]

use ical::tree::{
    cst::IcalCst,
    merge::{IcalMerge, IcalMergeAction, IcalMergeReason, IcalMergeReport},
};

/// The base calendar every case starts from: one organised event, folded on a
/// line nobody edits, so byte preservation is visible.
const BASE: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     PRODID:-//Example//EN\r\n\
     BEGIN:VEVENT\r\n\
     UID:event-1@example.com\r\n\
     DTSTAMP:20260101T000000Z\r\n\
     DTSTART:20260105T090000Z\r\n\
     SUMMARY:Weekly sync\r\n\
     DESCRIPTION:A description long enough that the parser had to fold it acro\r\n \
     ss two physical lines\r\n\
     LOCATION:Room A\r\n\
     ORGANIZER:mailto:chair@example.com\r\n\
     ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:ada@example.com\r\n\
     CATEGORIES:work,weekly\r\n\
     END:VEVENT\r\n\
     END:VCALENDAR\r\n";

/// Merge three calendars given as wire bytes.
fn merge<'a>(base: &'a str, left: &'a str, right: &'a str) -> IcalMergeReport<'a> {
    let base = Box::leak(Box::new(IcalCst::parse(base).expect("a readable base")));
    let left = Box::leak(Box::new(IcalCst::parse(left).expect("a readable left")));
    let right = Box::leak(Box::new(IcalCst::parse(right).expect("a readable right")));

    IcalMerge {
        base,
        left,
        right,
        right_speaks_for: None,
    }
    .merge()
}

/// The merged calendar's bytes.
fn bytes(report: &IcalMergeReport<'_>) -> String {
    String::from_utf8(report.merged.to_bytes()).expect("valid UTF-8")
}

/// The base with one line replaced.
fn edited(from: &str, to: &str) -> String {
    assert!(BASE.contains(from), "the base does not hold `{from}`");
    BASE.replace(from, to)
}

#[test]
fn takes_both_sides_edits_when_they_fall_on_different_properties() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly sync (moved)");
    let right = edited("LOCATION:Room A", "LOCATION:Room B");

    let report = merge(BASE, &left, &right);
    let merged = bytes(&report);

    assert!(merged.contains("SUMMARY:Weekly sync (moved)"));
    assert!(merged.contains("LOCATION:Room B"));
    assert!(report.conflicts.is_empty());

    // NOTE: The folded description was touched by neither side, so it comes out
    // folded exactly where it was.
    assert!(merged.contains("fold it acro\r\n ss two physical lines"));
}

#[test]
fn reports_a_conflict_rather_than_letting_one_side_win_silently() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Sprint sync");
    let right = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly standup");

    let report = merge(BASE, &left, &right);

    assert_eq!(report.conflicts.len(), 1);
    assert!(matches!(
        report.conflicts[0].reason,
        IcalMergeReason::Divergent(_)
    ));

    // NOTE: The left side's outcome is the one kept, and the right side's is
    // reported rather than dropped on the floor.
    assert!(bytes(&report).contains("SUMMARY:Sprint sync"));
    assert!(matches!(
        &report.conflicts[0].right,
        IcalMergeAction::ValueChanged { new, .. } if new_text(new) == "Weekly standup"
    ));
}

/// The text of a value, for the value shapes these cases use.
fn new_text<'v>(value: &'v ical::value::IcalValue<'_>) -> &'v str {
    match value {
        ical::value::IcalValue::Text(text) => &text.0,
        _ => "",
    }
}

#[test]
fn keeps_the_data_when_a_removal_meets_an_update() {
    let left = BASE.replace("LOCATION:Room A\r\n", "");
    let right = edited("LOCATION:Room A", "LOCATION:Room B");

    let report = merge(BASE, &left, &right);

    // NOTE: One side says the location is gone and the other says what it now
    // is. Keeping both is impossible; keeping the data is the lesser loss, and
    // the collision is still reported.
    assert!(bytes(&report).contains("LOCATION:Room B"));
    assert_eq!(report.conflicts.len(), 1);
}

#[test]
fn merges_list_items_as_a_set() {
    let left = edited("CATEGORIES:work,weekly", "CATEGORIES:work,weekly,team");
    let right = edited("CATEGORIES:work,weekly", "CATEGORIES:work,urgent");

    let report = merge(BASE, &left, &right);
    let merged = bytes(&report);

    // NOTE: Both additions apply and the removal applies, so two sides editing
    // one list never collide.
    assert!(merged.contains("CATEGORIES:work,team,urgent"));
    assert!(report.conflicts.is_empty());
}

#[test]
fn merges_a_parameter_one_side_changed() {
    let left = BASE.to_owned();
    let right = edited("PARTSTAT=NEEDS-ACTION", "PARTSTAT=ACCEPTED");

    let report = merge(BASE, &left, &right);

    assert!(bytes(&report).contains("PARTSTAT=ACCEPTED"));
    assert_eq!(report.left, []);
    assert_eq!(report.right.len(), 1);
    assert!(matches!(
        report.right[0],
        IcalMergeAction::ParamChanged { .. }
    ));
}

#[test]
fn matches_an_override_by_its_recurrence_id_rather_than_its_position() {
    let series = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:series@example.com\r\n\
         DTSTART:20260105T090000Z\r\n\
         RRULE:FREQ=WEEKLY\r\n\
         SUMMARY:Standup\r\n\
         END:VEVENT\r\n\
         BEGIN:VEVENT\r\n\
         UID:series@example.com\r\n\
         RECURRENCE-ID:20260112T090000Z\r\n\
         DTSTART:20260112T100000Z\r\n\
         SUMMARY:Standup\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    // NOTE: The right side rewrote the file with the override first. Matching
    // by position would pair the override with the series and read the
    // difference as an edit of both.
    let reordered = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:series@example.com\r\n\
         RECURRENCE-ID:20260112T090000Z\r\n\
         DTSTART:20260112T100000Z\r\n\
         SUMMARY:Standup, moved\r\n\
         END:VEVENT\r\n\
         BEGIN:VEVENT\r\n\
         UID:series@example.com\r\n\
         DTSTART:20260105T090000Z\r\n\
         RRULE:FREQ=WEEKLY\r\n\
         SUMMARY:Standup\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    let report = merge(series, series, reordered);

    assert_eq!(report.right.len(), 1);
    assert!(matches!(
        report.right[0],
        IcalMergeAction::ValueChanged { .. }
    ));

    let merged = bytes(&report);
    assert!(merged.contains("SUMMARY:Standup, moved"));
    assert!(merged.contains("RRULE:FREQ=WEEKLY"));
}

#[test]
fn reports_a_rule_change_against_an_instance_change() {
    let series = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:series@example.com\r\n\
         DTSTART:20260105T090000Z\r\n\
         RRULE:FREQ=WEEKLY\r\n\
         END:VEVENT\r\n\
         BEGIN:VEVENT\r\n\
         UID:series@example.com\r\n\
         RECURRENCE-ID:20260112T090000Z\r\n\
         DTSTART:20260112T100000Z\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    let left = series.replace("RRULE:FREQ=WEEKLY", "RRULE:FREQ=DAILY");
    let right = series.replace("DTSTART:20260112T100000Z", "DTSTART:20260112T110000Z");

    let report = merge(series, &left, &right);

    // NOTE: Neither side is wrong and both survive, but the rule that moved may
    // have moved the ground the override stood on.
    assert_eq!(report.conflicts.len(), 1);
    assert!(matches!(
        report.conflicts[0].reason,
        IcalMergeReason::Recurrence(_)
    ));

    let merged = bytes(&report);
    assert!(merged.contains("RRULE:FREQ=DAILY"));
    assert!(merged.contains("DTSTART:20260112T110000Z"));
}

#[test]
fn adds_and_removes_whole_components() {
    let alarm = "BEGIN:VALARM\r\nTRIGGER:-PT15M\r\nACTION:DISPLAY\r\nEND:VALARM\r\n";
    let left = BASE.to_owned();
    let right = BASE.replace("END:VEVENT", &format!("{alarm}END:VEVENT"));

    let report = merge(BASE, &left, &right);

    assert_eq!(report.right.len(), 1);
    assert!(matches!(
        report.right[0],
        IcalMergeAction::ComponentAdded { .. }
    ));
    assert!(bytes(&report).contains("TRIGGER:-PT15M"));

    let removed = merge(&right, &right, BASE);
    assert!(matches!(
        removed.right[0],
        IcalMergeAction::ComponentRemoved { .. }
    ));
    assert!(!bytes(&removed).contains("TRIGGER:-PT15M"));
}

#[test]
fn refuses_a_change_the_right_side_has_no_authority_over() {
    let base = Box::leak(Box::new(IcalCst::parse(BASE).expect("a readable base")));
    let left = Box::leak(Box::new(IcalCst::parse(BASE).expect("a readable left")));

    let edit = edited("DTSTART:20260105T090000Z", "DTSTART:20260105T100000Z");
    let right = Box::leak(Box::new(IcalCst::parse(&edit).expect("a readable right")));

    let report = IcalMerge {
        base,
        left,
        right,
        right_speaks_for: Some("mailto:ada@example.com".into()),
    }
    .merge();

    // NOTE: Ada is an attendee, and the start of a meeting is the organiser's
    // to set (RFC 5546 3.2).
    assert_eq!(report.conflicts.len(), 1);
    assert_eq!(report.conflicts[0].reason, IcalMergeReason::Authority);
    assert!(bytes(&report).contains("DTSTART:20260105T090000Z"));
}

#[test]
fn lets_an_attendee_set_what_is_theirs() {
    let base = Box::leak(Box::new(IcalCst::parse(BASE).expect("a readable base")));
    let left = Box::leak(Box::new(IcalCst::parse(BASE).expect("a readable left")));

    let edit = edited("PARTSTAT=NEEDS-ACTION", "PARTSTAT=ACCEPTED");
    let right = Box::leak(Box::new(IcalCst::parse(&edit).expect("a readable right")));

    let report = IcalMerge {
        base,
        left,
        right,
        right_speaks_for: Some("mailto:ada@example.com".into()),
    }
    .merge();

    assert!(report.conflicts.is_empty());
    assert!(bytes(&report).contains("PARTSTAT=ACCEPTED"));
}

#[test]
fn changes_nothing_when_neither_side_changed_anything() {
    let report = merge(BASE, BASE, BASE);

    assert_eq!(report.left, []);
    assert_eq!(report.right, []);
    assert!(report.conflicts.is_empty());
    assert_eq!(bytes(&report), BASE);
}
