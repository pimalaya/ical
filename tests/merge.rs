//! # Three-way merge
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

    IcalMerge { base, left, right }.merge()
}

/// The merged calendar's bytes.
fn bytes(report: &IcalMergeReport<'_>) -> String {
    String::from_utf8(report.merged.to_bytes()).expect("valid UTF-8")
}

/// The merged calendar read back, the law that a merge never emits bytes its
/// own parser refuses.
fn reparsed(merged: &str) -> String {
    let held = IcalCst::parse(merged.as_bytes()).expect("a readable merge");
    String::from_utf8(held.to_bytes()).expect("valid UTF-8")
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
        report.conflicts[0].left,
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
    let removed = BASE.replace("LOCATION:Room A\r\n", "");
    let updated = edited("LOCATION:Room A", "LOCATION:Room B");

    // NOTE: one side says the location is gone and the other says what it now
    // is. Keeping both is impossible; keeping the data is the lesser loss,
    // whichever side removed and whichever side updated, so the left side
    // winning a collision does not reach this case.
    let right_updated = merge(BASE, &removed, &updated);
    assert!(bytes(&right_updated).contains("LOCATION:Room B"));
    assert_eq!(right_updated.conflicts.len(), 1);

    let left_updated = merge(BASE, &updated, &removed);
    assert!(bytes(&left_updated).contains("LOCATION:Room B"));
    assert_eq!(left_updated.conflicts.len(), 1);
}

#[test]
fn leaves_a_property_only_one_side_touched_to_that_side() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly sync (moved)");
    let right = edited("LOCATION:Room A", "LOCATION:Room B");

    // NOTE: nobody is contesting either property, so neither side winning a
    // collision has anything to decide here.
    let merged = bytes(&merge(BASE, &left, &right));

    assert!(merged.contains("SUMMARY:Weekly sync (moved)"));
    assert!(merged.contains("LOCATION:Room B"));
}

#[test]
fn keeps_an_untouched_fold_through_a_collision() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Sprint sync");
    let right = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly standup");

    let merged = bytes(&merge(BASE, &left, &right));

    // NOTE: settling a collision moves a value, never the bytes of a line
    // neither side wrote to.
    assert!(merged.contains("fold it acro\r\n ss two physical lines"));
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

/// A base carrying the RFC 5545 section 3.2.1 alternate representation, whose
/// quoted parameter value holds a colon.
const ALTREP: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     BEGIN:VEVENT\r\n\
     UID:event-1@example.com\r\n\
     DTSTART:20260105T090000Z\r\n\
     DESCRIPTION;ALTREP=\"cid:part1.0001@example.org\":Meeting notes\r\n\
     END:VEVENT\r\n\
     END:VCALENDAR\r\n";

#[test]
fn tells_a_quoted_parameter_apart_from_the_value_behind_it() {
    let left = ALTREP.replace(":Meeting notes", ":Meeting minutes");
    let right = ALTREP.replace("part1.0001", "part2.0002");

    let report = merge(ALTREP, &left, &right);

    // NOTE: The head ends at the colon outside the quotes, so one side wrote
    // the value and the other the parameter: two fields, nothing contested. A
    // head cut inside the quotes folds both edits into one value and invents a
    // collision.
    assert!(bytes(&report).contains("ALTREP=\"cid:part2.0002@example.org\":Meeting minutes"));
    assert!(report.conflicts.is_empty());
    assert_eq!(report.left.len(), 1);
    assert!(matches!(
        report.left[0],
        IcalMergeAction::ValueChanged { .. }
    ));
    assert_eq!(report.right.len(), 1);
    assert!(matches!(
        report.right[0],
        IcalMergeAction::ParamChanged { .. }
    ));
}

#[test]
fn requoting_a_parameter_is_not_a_change() {
    // NOTE: The quotes RFC 5545 section 3.1 wraps a parameter value in are the
    // grammar's, not the value's, so a server that adds or drops a pair around
    // an unchanged value has changed nothing to report or collide over.
    let left = edited("PARTSTAT=NEEDS-ACTION", "PARTSTAT=\"NEEDS-ACTION\"");

    let report = merge(BASE, &left, BASE);

    assert_eq!(bytes(&report), left);
    assert!(report.conflicts.is_empty());
    assert!(report.right.is_empty());
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
        report.conflicts[0].left,
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

/// An event whose `CATEGORIES` carries a language and a transfer encoding, the
/// shape the merge fuzzer reduced to: the parameter action lands in the head,
/// where a stray byte ends the line.
const ENCODED_CATEGORIES: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     BEGIN:VEVENT\r\n\
     UID:event-1@example.com\r\n\
     CATEGORIES;LANGUAGE=en;ENCODING=QUOTED-PRINTABLE:TMS Dates\r\n\
     END:VEVENT\r\n\
     END:VCALENDAR\r\n";

#[test]
fn writes_a_replayed_parameter_as_the_side_that_wrote_it_spelt_it() {
    // NOTE: The parameter value holds a `\n`, which decoding resolves to a
    // newline and encoding does not put back, so a re-encoded parameter used to
    // end the line in the middle of its own head.
    let right = ENCODED_CATEGORIES.replace(
        "CATEGORIES;LANGUAGE=en;ENCODING=QUOTED-PRINTABLE:TMS Dates",
        r"CATEGORIES;LANGUAGE=en\n2:Reviews",
    );

    let report = merge(ENCODED_CATEGORIES, ENCODED_CATEGORIES, &right);
    let merged = bytes(&report);

    assert!(merged.contains(concat!(r"CATEGORIES;LANGUAGE=en\n2:Reviews", "\r\n")));
    assert_eq!(reparsed(&merged), merged);
}

#[test]
fn keeps_a_replayed_list_item_on_its_line_across_escaping_rules() {
    // NOTE: The item decodes to a newline under the right side's rules, and
    // vCalendar 1.0, which the baseline side is written in, has no escape for
    // one: written raw it would end the line.
    let base = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nCATEGORIES:work\r\nEND:VCALENDAR\r\n";
    let left = base.replace("VERSION:2.0", "VERSION:1.0");
    let right = base.replace("CATEGORIES:work", r"CATEGORIES:work,two\nlines");

    let merged = bytes(&merge(base, &left, &right));

    assert!(merged.contains(concat!(r"CATEGORIES:work,two\nlines", "\r\n")));
    assert_eq!(reparsed(&merged), merged);
}

/// A value differing only past its first `;` is still a changed value.
///
/// The diff used to compare the decoded values, and a text value decodes its
/// first `;`-component alone, so two lines saying different things read alike
/// and the right side's edit reached neither the report nor the bytes.
#[test]
fn sees_an_edit_past_the_first_semicolon_of_a_text_value() {
    let base = edited("LOCATION:Room A", "LOCATION:Room A;floor 2");
    let right = edited("LOCATION:Room A", "LOCATION:Room A;floor 9");

    let report = merge(&base, &base, &right);

    assert_eq!(report.right.len(), 1, "{:?}", report.right);
    assert!(
        bytes(&report).contains("LOCATION:Room A;floor 9\r\n"),
        "{}",
        bytes(&report),
    );
}

/// A list is a multiset, so dropping one of two equal items is a removal.
///
/// Membership alone left the other copy standing in for the one taken away,
/// so the removal was reported as nothing and never applied.
#[test]
fn removes_one_of_two_equal_list_items() {
    let base = edited("CATEGORIES:work,weekly", "CATEGORIES:work,work,weekly");
    let right = edited("CATEGORIES:work,weekly", "CATEGORIES:work,weekly");

    let report = merge(&base, &base, &right);

    assert_eq!(report.right.len(), 1, "{:?}", report.right);
    assert!(
        bytes(&report).contains("CATEGORIES:work,weekly\r\n"),
        "{}",
        bytes(&report),
    );
}

/// A line the left side inserted moves every line after it, so the right
/// side's edit must follow its property rather than its old position.
///
/// The shift was read off the left side's removals alone, so an insertion
/// renumbered nothing and the edit landed on the line before the one meant:
/// one property was overwritten and another left stale, with no conflict.
#[test]
fn addresses_the_right_line_when_the_left_side_inserted_one() {
    let base = edited("LOCATION:Room A", "COMMENT:one\r\nCOMMENT:two");
    let left = edited(
        "LOCATION:Room A",
        "COMMENT:zero\r\nCOMMENT:one\r\nCOMMENT:two",
    );
    let right = edited("LOCATION:Room A", "COMMENT:one\r\nCOMMENT:two edited");

    let report = merge(&base, &left, &right);
    let merged = bytes(&report);

    assert!(merged.contains("COMMENT:zero\r\n"), "{merged}");
    assert!(merged.contains("COMMENT:one\r\n"), "{merged}");
    assert!(merged.contains("COMMENT:two edited\r\n"), "{merged}");
    assert!(
        !merged.contains("COMMENT:two\r\n"),
        "the stale line is gone: {merged}",
    );
}

/// A property the left side removed and the right side edited twice comes
/// back once, not once per edit.
///
/// The restored line is the right side's own, bytes and all, so every action
/// on that property is already in it. Pushing per action left one copy each.
#[test]
fn restores_a_removed_property_once_however_many_edits_it_carries() {
    let base = edited("LOCATION:Room A", "LOCATION;LANGUAGE=en:Room A");
    let left = edited("LOCATION:Room A", "SUMMARY:Weekly sync");
    let right = edited("LOCATION:Room A", "LOCATION;LANGUAGE=fr:Salle B");

    let report = merge(&base, &left, &right);
    let merged = bytes(&report);

    assert_eq!(
        merged.matches("LOCATION").count(),
        1,
        "one restored line, not one per action: {merged}",
    );
    assert!(merged.contains("LOCATION;LANGUAGE=fr:Salle B"), "{merged}");
}

/// Retyping a value contests the other side's items, rather than both landing.
///
/// `VALUE` declares how the value is read, so a side switching it and a side
/// adding an item under the old type cannot both be right: the result would
/// declare one type and carry items of another (RFC 5545 section 3.8.5.2).
#[test]
fn retyping_a_value_collides_with_the_other_sides_items() {
    let base = edited("LOCATION:Room A", "RDATE;VALUE=DATE-TIME:20260105T090000Z");
    let left = edited(
        "LOCATION:Room A",
        "RDATE;VALUE=DATE-TIME:20260105T090000Z,20260112T090000Z",
    );
    let right = edited(
        "LOCATION:Room A",
        "RDATE;VALUE=PERIOD:20260105T090000Z/PT1H",
    );

    let report = merge(&base, &left, &right);

    assert!(
        !report.conflicts.is_empty(),
        "the retype is contested: {:?}",
        report.conflicts,
    );
}

#[test]
fn sees_a_parameter_edit_past_the_value_its_decode_reads() {
    // A single-valued parameter decodes its first value alone, so comparing
    // the decoded parameters hides an edit to anything after it: the change
    // is never reported and the right side's edit is silently dropped.
    let base = edited(
        "ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:ada@example.com",
        "ATTENDEE;CN=Ada,Lovelace:mailto:ada@example.com",
    );
    let right = base.replace("CN=Ada,Lovelace", "CN=Ada,Byron");

    let report = merge(&base, &base, &right);

    assert!(
        bytes(&report).contains("CN=Ada,Byron"),
        "{}",
        bytes(&report)
    );
    assert_eq!(report.right.len(), 1);
    assert!(matches!(
        report.right[0],
        IcalMergeAction::ParamChanged { .. }
    ));
}

/// Two sides that wrote different bytes did not perform one act, however
/// alike the two read.
///
/// `\N` and `\n` both unescape to a line break (RFC 5545 section 3.3.11), so
/// the two actions decoded equal and the right side's edit was taken for the
/// left side's own: it was skipped as already made and the difference between
/// the two spellings was never said out loud.
#[test]
fn spelling_a_value_two_ways_is_no_agreement() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Sprint\\nsync");
    let right = edited("SUMMARY:Weekly sync", "SUMMARY:Sprint\\Nsync");

    let report = merge(BASE, &left, &right);

    assert_eq!(report.conflicts.len(), 1, "{:?}", report.conflicts);
    assert!(matches!(
        report.conflicts[0].left,
        IcalMergeReason::Divergent(_),
    ));

    // NOTE: the left side is the one being merged into, so it keeps its own
    // spelling and the merged calendar is its bytes, untouched.
    assert_eq!(bytes(&report), left);
}

/// A parameter the specification gives no order is a set, so two sides
/// writing one list in two orders wrote one parameter.
///
/// `DELEGATED-TO` holds a list RFC 5545 section 3.2.5 gives no order, so
/// neither arrangement says anything the other does not.
#[test]
fn writing_an_unordered_list_parameter_in_two_orders_is_agreement() {
    let left = edited(
        "ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:ada@example.com",
        "ATTENDEE;PARTSTAT=NEEDS-ACTION;DELEGATED-TO=\"mailto:bob@example.com\",\"mailto:cyd@example.com\":mailto:ada@example.com",
    );
    let right = edited(
        "ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:ada@example.com",
        "ATTENDEE;PARTSTAT=NEEDS-ACTION;DELEGATED-TO=\"mailto:cyd@example.com\",\"mailto:bob@example.com\":mailto:ada@example.com",
    );

    let report = merge(BASE, &left, &right);

    assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
    assert_eq!(bytes(&report), left);
}
