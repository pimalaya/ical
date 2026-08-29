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
    merge::{IcalMerge, IcalMergeAction, IcalMergeReason, IcalMergeReport, IcalMergeSide},
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

/// Merge three calendars given as wire bytes, stating no preference.
fn merge<'a>(base: &'a str, left: &'a str, right: &'a str) -> IcalMergeReport<'a> {
    merge_preferring(base, left, right, IcalMergeSide::default())
}

/// The same, stating which side wins a field both sides wrote a value into.
fn merge_preferring<'a>(
    base: &'a str,
    left: &'a str,
    right: &'a str,
    prefer: IcalMergeSide,
) -> IcalMergeReport<'a> {
    let base = Box::leak(Box::new(IcalCst::parse(base).expect("a readable base")));
    let left = Box::leak(Box::new(IcalCst::parse(left).expect("a readable left")));
    let right = Box::leak(Box::new(IcalCst::parse(right).expect("a readable right")));

    IcalMerge {
        base,
        left,
        right,
        right_speaks_for: None,
        prefer,
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
fn carries_the_right_sides_value_when_the_right_side_is_preferred() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Sprint sync");
    let right = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly standup");

    let report = merge_preferring(BASE, &left, &right, IcalMergeSide::Right);

    assert!(bytes(&report).contains("SUMMARY:Weekly standup"));

    // NOTE: The winner changed, not the report: both actions are still named,
    // the right side's on the conflict and the left side's on its reason.
    assert_eq!(report.conflicts.len(), 1);
    assert!(matches!(
        &report.conflicts[0].right,
        IcalMergeAction::ValueChanged { new, .. } if new_text(new) == "Weekly standup"
    ));
    assert!(matches!(
        &report.conflicts[0].reason,
        IcalMergeReason::Divergent(IcalMergeAction::ValueChanged { new, .. })
            if new_text(new) == "Sprint sync"
    ));
}

#[test]
fn keeps_the_left_sides_value_when_the_left_side_is_preferred() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Sprint sync");
    let right = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly standup");

    let stated = merge_preferring(BASE, &left, &right, IcalMergeSide::Left);
    let silent = merge(BASE, &left, &right);

    // NOTE: Preferring the left side is what a merge has always done, so
    // saying it out loud has to give what saying nothing gives.
    assert!(bytes(&stated).contains("SUMMARY:Sprint sync"));
    assert_eq!(bytes(&stated), bytes(&silent));
    assert_eq!(stated.conflicts, silent.conflicts);
}

#[test]
fn keeps_the_data_when_a_removal_meets_an_update() {
    let removed = BASE.replace("LOCATION:Room A\r\n", "");
    let updated = edited("LOCATION:Room A", "LOCATION:Room B");

    for prefer in [IcalMergeSide::Left, IcalMergeSide::Right] {
        // NOTE: One side says the location is gone and the other says what it
        // now is. Keeping both is impossible; keeping the data is the lesser
        // loss, and that is not the preference's to invert, whichever side
        // removed and whichever side updated.
        let right_updated = merge_preferring(BASE, &removed, &updated, prefer);
        assert!(bytes(&right_updated).contains("LOCATION:Room B"));
        assert_eq!(right_updated.conflicts.len(), 1);

        let left_updated = merge_preferring(BASE, &updated, &removed, prefer);
        assert!(bytes(&left_updated).contains("LOCATION:Room B"));
        assert_eq!(left_updated.conflicts.len(), 1);
    }
}

#[test]
fn leaves_a_property_only_one_side_touched_to_that_side() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly sync (moved)");
    let right = edited("LOCATION:Room A", "LOCATION:Room B");

    for prefer in [IcalMergeSide::Left, IcalMergeSide::Right] {
        let merged = bytes(&merge_preferring(BASE, &left, &right, prefer));

        // NOTE: Nobody is contesting either property, so there is nothing for
        // a preference to decide.
        assert!(merged.contains("SUMMARY:Weekly sync (moved)"));
        assert!(merged.contains("LOCATION:Room B"));
    }
}

#[test]
fn keeps_an_untouched_fold_under_either_preference() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Sprint sync");
    let right = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly standup");

    for prefer in [IcalMergeSide::Left, IcalMergeSide::Right] {
        let merged = bytes(&merge_preferring(BASE, &left, &right, prefer));

        // NOTE: The preference moves a value, never the bytes of a line
        // neither side wrote to.
        assert!(merged.contains("fold it acro\r\n ss two physical lines"));
    }
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
    let edit = edited("DTSTART:20260105T090000Z", "DTSTART:20260105T100000Z");

    for prefer in [IcalMergeSide::Left, IcalMergeSide::Right] {
        let report = speaking_for(BASE, BASE, &edit, "mailto:ada@example.com", prefer);

        // NOTE: Ada is an attendee, and the start of a meeting is the
        // organiser's to set (RFC 5546 3.2). Preferring the side a change came
        // from does not grant it the authority to make that change.
        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.conflicts[0].reason, IcalMergeReason::Authority);
        assert!(bytes(&report).contains("DTSTART:20260105T090000Z"));
    }
}

#[test]
fn lets_an_attendee_set_what_is_theirs() {
    let edit = edited("PARTSTAT=NEEDS-ACTION", "PARTSTAT=ACCEPTED");
    let report = speaking_for(
        BASE,
        BASE,
        &edit,
        "mailto:ada@example.com",
        IcalMergeSide::Left,
    );

    assert!(report.conflicts.is_empty());
    assert!(bytes(&report).contains("PARTSTAT=ACCEPTED"));
}

/// A base holding a meeting Ada was invited to and a task nobody organises.
const INVITED_AND_OWN: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     BEGIN:VEVENT\r\n\
     UID:event-1@example.com\r\n\
     DTSTART:20260105T090000Z\r\n\
     SUMMARY:Weekly sync\r\n\
     ORGANIZER:mailto:chair@example.com\r\n\
     ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:ada@example.com\r\n\
     END:VEVENT\r\n\
     BEGIN:VEVENT\r\n\
     UID:event-2@example.com\r\n\
     DTSTART:20260106T090000Z\r\n\
     SUMMARY:Write the report\r\n\
     END:VEVENT\r\n\
     END:VCALENDAR\r\n";

#[test]
fn lets_a_judged_side_win_a_collision_it_is_allowed_to_make() {
    let left = INVITED_AND_OWN.replace("SUMMARY:Write the report", "SUMMARY:Draft the report");
    let right = INVITED_AND_OWN
        .replace(
            "SUMMARY:Write the report",
            "SUMMARY:Write the quarterly report",
        )
        .replace("PARTSTAT=NEEDS-ACTION", "PARTSTAT=ACCEPTED");

    let report = speaking_for(
        INVITED_AND_OWN,
        &left,
        &right,
        "mailto:ada@example.com",
        IcalMergeSide::Right,
    );
    let merged = bytes(&report);

    // NOTE: Ada is judged, so she answers the invitation rather than moving the
    // meeting, and being judged no longer costs her the summary of her own
    // task. Only the summary is contested, and only the summary is reported.
    assert!(merged.contains("PARTSTAT=ACCEPTED"));
    assert!(merged.contains("SUMMARY:Write the quarterly report"));
    assert_eq!(report.conflicts.len(), 1);
    assert!(matches!(
        report.conflicts[0].reason,
        IcalMergeReason::Divergent(_)
    ));
}

/// Merge three calendars with the right side edited on someone's behalf.
fn speaking_for<'a>(
    base: &'a str,
    left: &'a str,
    right: &'a str,
    speaker: &'a str,
    prefer: IcalMergeSide,
) -> IcalMergeReport<'a> {
    let base = Box::leak(Box::new(IcalCst::parse(base).expect("a readable base")));
    let left = Box::leak(Box::new(IcalCst::parse(left).expect("a readable left")));
    let right = Box::leak(Box::new(IcalCst::parse(right).expect("a readable right")));

    IcalMerge {
        base,
        left,
        right,
        right_speaks_for: Some(speaker.into()),
        prefer,
    }
    .merge()
}

#[test]
fn changes_nothing_when_neither_side_changed_anything() {
    let report = merge(BASE, BASE, BASE);

    assert_eq!(report.left, []);
    assert_eq!(report.right, []);
    assert!(report.conflicts.is_empty());
    assert_eq!(bytes(&report), BASE);
}

/// A `VTIMEZONE` whose `STANDARD` is written before its `DAYLIGHT`, so the
/// second observance is not the first child of its parent.
const ZONED: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     BEGIN:VTIMEZONE\r\n\
     TZID:Europe/Paris\r\n\
     BEGIN:STANDARD\r\n\
     DTSTART:19701025T030000\r\n\
     TZOFFSETFROM:+0200\r\n\
     TZOFFSETTO:+0100\r\n\
     END:STANDARD\r\n\
     BEGIN:DAYLIGHT\r\n\
     DTSTART:19700329T020000\r\n\
     TZOFFSETFROM:+0100\r\n\
     TZOFFSETTO:+0200\r\n\
     END:DAYLIGHT\r\n\
     END:VTIMEZONE\r\n\
     END:VCALENDAR\r\n";

#[test]
fn applies_a_change_to_a_component_that_is_not_the_first_child() {
    let right = ZONED.replace(
        "BEGIN:DAYLIGHT\r\nDTSTART:19700329T020000",
        "BEGIN:DAYLIGHT\r\nDTSTART:19700330T020000",
    );

    let report = merge(ZONED, ZONED, &right);

    // NOTE: A component's position is counted among its same-named siblings,
    // so the `STANDARD` written before the `DAYLIGHT` does not shift it. The
    // left side changed nothing, so the right side's one change has to land,
    // and land unreported.
    assert_eq!(report.right.len(), 1);
    assert!(report.conflicts.is_empty());
    assert!(bytes(&report).contains("DTSTART:19700330T020000"));

    // NOTE: The observance the right side did not touch is untouched, which is
    // what tells a change that landed from a change that landed anywhere.
    assert!(bytes(&report).contains("BEGIN:STANDARD\r\nDTSTART:19701025T030000"));
}

/// An event with three attendees, so a removal has same-named siblings after
/// it to renumber.
const THREE_ATTENDEES: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     BEGIN:VEVENT\r\n\
     UID:event-1@example.com\r\n\
     ATTENDEE;CN=Ada:mailto:ada@example.com\r\n\
     ATTENDEE;CN=Bob:mailto:bob@example.com\r\n\
     ATTENDEE;CN=Cyd:mailto:cyd@example.com\r\n\
     END:VEVENT\r\n\
     END:VCALENDAR\r\n";

#[test]
fn applies_every_removal_from_one_group() {
    let none = THREE_ATTENDEES
        .replace("ATTENDEE;CN=Ada:mailto:ada@example.com\r\n", "")
        .replace("ATTENDEE;CN=Bob:mailto:bob@example.com\r\n", "")
        .replace("ATTENDEE;CN=Cyd:mailto:cyd@example.com\r\n", "");

    let report = merge(THREE_ATTENDEES, THREE_ATTENDEES, &none);

    assert_eq!(report.right.len(), 3);
    assert!(report.conflicts.is_empty());
    assert!(!bytes(&report).contains("ATTENDEE"));
}

#[test]
fn removes_the_members_the_side_removed_rather_than_the_ones_after_them() {
    let last = THREE_ATTENDEES
        .replace("ATTENDEE;CN=Ada:mailto:ada@example.com\r\n", "")
        .replace("ATTENDEE;CN=Bob:mailto:bob@example.com\r\n", "");

    let report = merge(THREE_ATTENDEES, THREE_ATTENDEES, &last);
    let merged = bytes(&report);

    // NOTE: Asserting the count alone would pass a merge that kept Bob and
    // dropped Cyd, which is what a replay in diff order does: the first
    // removal renumbers the two after it.
    assert!(merged.contains("ATTENDEE;CN=Cyd:mailto:cyd@example.com"));
    assert!(!merged.contains("CN=Ada"));
    assert!(!merged.contains("CN=Bob"));
}

/// An event with three alarms, the component peer of the case above: a
/// `VALARM` has no `UID`, so it is addressed by its position too.
const THREE_ALARMS: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     BEGIN:VEVENT\r\n\
     UID:event-1@example.com\r\n\
     BEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT10M\r\nEND:VALARM\r\n\
     BEGIN:VALARM\r\nACTION:AUDIO\r\nTRIGGER:-PT20M\r\nEND:VALARM\r\n\
     BEGIN:VALARM\r\nACTION:EMAIL\r\nTRIGGER:-PT30M\r\nEND:VALARM\r\n\
     END:VEVENT\r\n\
     END:VCALENDAR\r\n";

#[test]
fn applies_every_removal_from_one_group_of_components() {
    let none = THREE_ALARMS
        .replace(
            "BEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT10M\r\nEND:VALARM\r\n",
            "",
        )
        .replace(
            "BEGIN:VALARM\r\nACTION:AUDIO\r\nTRIGGER:-PT20M\r\nEND:VALARM\r\n",
            "",
        )
        .replace(
            "BEGIN:VALARM\r\nACTION:EMAIL\r\nTRIGGER:-PT30M\r\nEND:VALARM\r\n",
            "",
        );

    let report = merge(THREE_ALARMS, THREE_ALARMS, &none);

    assert_eq!(report.right.len(), 3);
    assert!(report.conflicts.is_empty());
    assert!(!bytes(&report).contains("VALARM"));
}
