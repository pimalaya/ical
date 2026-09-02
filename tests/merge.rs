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

/// What each reported collision lands on, in the order the report gives them.
fn collided(report: &IcalMergeReport<'_>) -> Vec<String> {
    report
        .conflicts
        .iter()
        .map(|pair| named(&pair.right))
        .collect()
}

/// What an action lands on: the name of a property, or the key of the
/// component a component-level action names.
fn named(action: &IcalMergeAction<'_>) -> String {
    match action {
        IcalMergeAction::ComponentAdded { at } | IcalMergeAction::ComponentRemoved { at } => {
            at.0.last()
                .map(|step| step.key.to_string())
                .unwrap_or_default()
        }
        IcalMergeAction::PropAdded { at, .. }
        | IcalMergeAction::PropRemoved { at, .. }
        | IcalMergeAction::ValueChanged { at, .. }
        | IcalMergeAction::ValueItemAdded { at, .. }
        | IcalMergeAction::ValueItemRemoved { at, .. }
        | IcalMergeAction::ParamAdded { at, .. }
        | IcalMergeAction::ParamRemoved { at, .. }
        | IcalMergeAction::ParamChanged { at, .. } => at.name.to_string(),
    }
}

/// Two properties each written differently on both sides are two reported
/// collisions, not the one a caller who has only ever seen a single-property
/// disagreement would expect.
#[test]
fn two_diverging_properties_are_reported_twice() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Sprint sync")
        .replace("LOCATION:Room A", "LOCATION:Room B");
    let right = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly standup")
        .replace("LOCATION:Room A", "LOCATION:Room C");

    let report = merge(BASE, &left, &right);

    assert_eq!(report.left.len(), 2);
    assert_eq!(report.right.len(), 2);
    assert_eq!(
        collided(&report),
        ["SUMMARY", "LOCATION"],
        "{:?}",
        report.conflicts
    );

    // NOTE: the left side wins both, so the count is the only trace the merged
    // calendar carries of the two values it did not keep.
    assert_eq!(bytes(&report), left);
}

/// Three properties each written differently on both sides are three reported
/// collisions, so the count follows the disagreement rather than saturating.
#[test]
fn three_diverging_properties_are_reported_three_times() {
    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Sprint sync")
        .replace("LOCATION:Room A", "LOCATION:Room B")
        .replace(
            "ORGANIZER:mailto:chair@example.com",
            "ORGANIZER:mailto:ada@example.com",
        );
    let right = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly standup")
        .replace("LOCATION:Room A", "LOCATION:Room C")
        .replace(
            "ORGANIZER:mailto:chair@example.com",
            "ORGANIZER:mailto:bob@example.com",
        );

    let report = merge(BASE, &left, &right);

    assert_eq!(
        collided(&report),
        ["SUMMARY", "LOCATION", "ORGANIZER"],
        "{:?}",
        report.conflicts,
    );
}

/// Edits that merge are not counted, so the number reports the disagreement
/// rather than the traffic.
///
/// Both sides wrote `SUMMARY` and `LOCATION` differently, and each also
/// touched fields the other left alone. Only the two contested ones are
/// reported, and every uncontested change lands.
#[test]
fn merged_edits_do_not_inflate_the_count() {
    let alarm = "BEGIN:VALARM\r\nTRIGGER:-PT15M\r\nACTION:DISPLAY\r\nEND:VALARM\r\n";

    let left = edited("SUMMARY:Weekly sync", "SUMMARY:Sprint sync")
        .replace("LOCATION:Room A", "LOCATION:Room B")
        .replace("DTSTAMP:20260101T000000Z", "DTSTAMP:20260102T000000Z");
    let right = edited("SUMMARY:Weekly sync", "SUMMARY:Weekly standup")
        .replace("LOCATION:Room A", "LOCATION:Room C")
        .replace("PARTSTAT=NEEDS-ACTION", "PARTSTAT=ACCEPTED")
        .replace("END:VEVENT", &format!("{alarm}END:VEVENT"));

    let report = merge(BASE, &left, &right);
    let merged = bytes(&report);

    assert_eq!(
        collided(&report),
        ["SUMMARY", "LOCATION"],
        "{:?}",
        report.conflicts
    );

    assert!(merged.contains("SUMMARY:Sprint sync"));
    assert!(merged.contains("LOCATION:Room B"));
    assert!(merged.contains("DTSTAMP:20260102T000000Z"));
    assert!(merged.contains("PARTSTAT=ACCEPTED"));
    assert!(merged.contains("TRIGGER:-PT15M"));
    assert_eq!(merged, reparsed(&merged));
}

/// A recurring event and the instance it overrides: one calendar object made
/// of two components, which a merge has to reconcile together.
const SERIES: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     PRODID:-//Example//EN\r\n\
     BEGIN:VEVENT\r\n\
     UID:series@example.com\r\n\
     DTSTAMP:20260101T000000Z\r\n\
     DTSTART:20260105T090000Z\r\n\
     RRULE:FREQ=WEEKLY\r\n\
     SUMMARY:Standup\r\n\
     LOCATION:Room A\r\n\
     END:VEVENT\r\n\
     BEGIN:VEVENT\r\n\
     UID:series@example.com\r\n\
     RECURRENCE-ID:20260112T090000Z\r\n\
     DTSTAMP:20260101T000000Z\r\n\
     DTSTART:20260112T100000Z\r\n\
     SUMMARY:Standup\r\n\
     LOCATION:Room B\r\n\
     END:VEVENT\r\n\
     END:VCALENDAR\r\n";

/// The number of components of one name the calendar holds.
fn components(merged: &str, name: &str) -> usize {
    merged.matches(&format!("BEGIN:{name}\r\n")).count()
}

/// A master edited on one side and an override edited on the other are one
/// object edited in two places, so both land and neither component is dropped.
#[test]
fn merges_a_master_edit_with_an_override_edit() {
    let left = SERIES.replace("LOCATION:Room A", "LOCATION:Room C");
    let right = SERIES.replace("LOCATION:Room B", "LOCATION:Room D");

    let report = merge(SERIES, &left, &right);
    let merged = bytes(&report);

    // NOTE: `LOCATION` says nothing about when the series happens, so the two
    // edits are not even reported against one another (RFC 5545 3.8.5).
    assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);

    assert!(merged.contains("LOCATION:Room C"));
    assert!(merged.contains("LOCATION:Room D"));
    assert!(merged.contains("RRULE:FREQ=WEEKLY"));
    assert!(merged.contains("RECURRENCE-ID:20260112T090000Z"));
    assert_eq!(components(&merged, "VEVENT"), 2);
}

/// One field of one override written differently on both sides is one
/// collision, reported once and not once per component of the object.
#[test]
fn one_field_of_an_override_collides_once() {
    let left = SERIES.replace("LOCATION:Room B", "LOCATION:Room C");
    let right = SERIES.replace("LOCATION:Room B", "LOCATION:Room D");

    let report = merge(SERIES, &left, &right);
    let merged = bytes(&report);

    assert_eq!(collided(&report), ["LOCATION"], "{:?}", report.conflicts);
    assert!(matches!(
        report.conflicts[0].left,
        IcalMergeReason::Divergent(_),
    ));

    // NOTE: the master carries a `LOCATION` too, and nobody touched it.
    assert!(merged.contains("LOCATION:Room A"));
    assert!(merged.contains("LOCATION:Room C"));
    assert_eq!(components(&merged, "VEVENT"), 2);
}

/// An override added on one side survives a master edited on the other, and
/// the object comes out holding all three components.
#[test]
fn merges_an_added_override_with_a_master_edit() {
    let added = "BEGIN:VEVENT\r\n\
         UID:series@example.com\r\n\
         RECURRENCE-ID:20260119T090000Z\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         DTSTART:20260119T110000Z\r\n\
         SUMMARY:Standup\r\n\
         LOCATION:Room E\r\n\
         END:VEVENT\r\n";

    let left = SERIES.replace("LOCATION:Room A", "LOCATION:Room C");
    let right = SERIES.replace("END:VCALENDAR", &format!("{added}END:VCALENDAR"));

    let report = merge(SERIES, &left, &right);
    let merged = bytes(&report);

    assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);

    assert!(merged.contains("LOCATION:Room C"));
    assert!(merged.contains("RECURRENCE-ID:20260119T090000Z"));
    assert!(merged.contains("LOCATION:Room E"));
    assert_eq!(components(&merged, "VEVENT"), 3);
    assert_eq!(merged, reparsed(&merged));
}

/// The master component of [`SERIES`], for the cases that delete it whole.
const MASTER: &str = "BEGIN:VEVENT\r\n\
     UID:series@example.com\r\n\
     DTSTAMP:20260101T000000Z\r\n\
     DTSTART:20260105T090000Z\r\n\
     RRULE:FREQ=WEEKLY\r\n\
     SUMMARY:Standup\r\n\
     LOCATION:Room A\r\n\
     END:VEVENT\r\n";

/// The overriding occurrence of [`SERIES`], for the cases that delete it
/// whole.
const OVERRIDE: &str = "BEGIN:VEVENT\r\n\
     UID:series@example.com\r\n\
     RECURRENCE-ID:20260112T090000Z\r\n\
     DTSTAMP:20260101T000000Z\r\n\
     DTSTART:20260112T100000Z\r\n\
     SUMMARY:Standup\r\n\
     LOCATION:Room B\r\n\
     END:VEVENT\r\n";

/// An override deleted on one side and edited on the other keeps the edited
/// occurrence, whichever side deleted it, and is reported once either way.
///
/// The rule that an update beats a removal is stated about the removal, not
/// about the granularity it happened at: a whole occurrence taken away is
/// exactly what the other side's edit was written against, so the component
/// comes back carrying that edit rather than the outcome following which of
/// the two sides happens to be `ours`.
#[test]
fn an_edited_override_outlives_a_deletion_from_either_side() {
    let deleted = SERIES.replace(OVERRIDE, "");
    let edited = SERIES.replace("LOCATION:Room B", "LOCATION:Room D");

    assert_eq!(components(&deleted, "VEVENT"), 1);

    // NOTE: the left side deleted, so the occurrence comes back as the right
    // side wrote it rather than the edit landing nowhere.
    let left_deleted = merge(SERIES, &deleted, &edited);
    let merged = bytes(&left_deleted);

    assert_eq!(
        left_deleted.conflicts.len(),
        1,
        "{:?}",
        left_deleted.conflicts,
    );
    assert!(matches!(
        left_deleted.conflicts[0].left,
        IcalMergeReason::Divergent(IcalMergeAction::ComponentRemoved { .. }),
    ));
    assert_eq!(components(&merged, "VEVENT"), 2);
    assert!(merged.contains("LOCATION:Room D"), "got: {merged}");
    assert!(
        merged.contains("RECURRENCE-ID:20260112T090000Z"),
        "{merged}"
    );
    assert_eq!(merged, reparsed(&merged));

    // NOTE: the left side edited, so the deletion is refused, and the two
    // sides come out holding the same occurrence.
    let right_deleted = merge(SERIES, &edited, &deleted);
    let merged = bytes(&right_deleted);

    assert_eq!(
        right_deleted.conflicts.len(),
        1,
        "{:?}",
        right_deleted.conflicts,
    );
    assert!(matches!(
        right_deleted.conflicts[0].right,
        IcalMergeAction::ComponentRemoved { .. },
    ));
    assert_eq!(components(&merged, "VEVENT"), 2);
    assert!(merged.contains("LOCATION:Room D"), "got: {merged}");
}

/// An occurrence one side deleted and the other left alone goes away, from
/// either side, with nothing to report.
///
/// A component comes back for the sake of an edit that would otherwise land
/// nowhere. An untouched occurrence has no such edit, so the deletion is a
/// change one side alone made and it simply applies.
#[test]
fn a_deleted_override_nobody_edited_goes_away() {
    let deleted = SERIES.replace(OVERRIDE, "");

    let left_deleted = merge(SERIES, &deleted, SERIES);
    let merged = bytes(&left_deleted);

    assert!(
        left_deleted.conflicts.is_empty(),
        "{:?}",
        left_deleted.conflicts
    );
    assert_eq!(components(&merged, "VEVENT"), 1);
    assert!(!merged.contains("RECURRENCE-ID"), "got: {merged}");

    let right_deleted = merge(SERIES, SERIES, &deleted);
    let merged = bytes(&right_deleted);

    assert!(
        right_deleted.conflicts.is_empty(),
        "{:?}",
        right_deleted.conflicts
    );
    assert_eq!(components(&merged, "VEVENT"), 1);
    assert!(!merged.contains("RECURRENCE-ID"), "got: {merged}");
}

/// An occurrence both sides deleted stays gone: they agreed, and agreement is
/// not a collision.
#[test]
fn an_override_both_sides_deleted_stays_gone() {
    let deleted = SERIES.replace(OVERRIDE, "");

    let report = merge(SERIES, &deleted, &deleted);
    let merged = bytes(&report);

    assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
    assert_eq!(components(&merged, "VEVENT"), 1);
    assert_eq!(merged, deleted);
}

/// A master deleted on one side and an override edited on the other are two
/// components, so the deletion applies and the edit applies with it.
///
/// Nothing was removed out from under the edit: the occurrence it lands on is
/// still there. What is said out loud is that the series the occurrence hangs
/// off is gone, which is a recurrence conflict rather than a divergence.
#[test]
fn a_deleted_master_leaves_the_edited_override_standing() {
    let deleted = SERIES.replace(MASTER, "");
    let edited = SERIES.replace("LOCATION:Room B", "LOCATION:Room D");

    for (left, right) in [(&deleted, &edited), (&edited, &deleted)] {
        let report = merge(SERIES, left, right);
        let merged = bytes(&report);

        assert_eq!(report.conflicts.len(), 1, "{:?}", report.conflicts);
        assert!(matches!(
            report.conflicts[0].left,
            IcalMergeReason::Recurrence(_),
        ));
        assert_eq!(components(&merged, "VEVENT"), 1);
        assert!(!merged.contains("RRULE:FREQ=WEEKLY"), "got: {merged}");
        assert!(merged.contains("LOCATION:Room D"), "got: {merged}");
    }
}

/// An event holding a reminder, so an edit can sit one component below the one
/// the other side deletes.
const REMINDED: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     BEGIN:VEVENT\r\n\
     UID:event-1@example.com\r\n\
     DTSTAMP:20260101T000000Z\r\n\
     SUMMARY:Weekly sync\r\n\
     BEGIN:VALARM\r\n\
     ACTION:DISPLAY\r\n\
     TRIGGER:-PT10M\r\n\
     END:VALARM\r\n\
     END:VEVENT\r\n\
     END:VCALENDAR\r\n";

/// An edit nested under a deleted component brings back the deleted component,
/// not only the one holding the edited line.
#[test]
fn an_edit_under_a_deleted_component_brings_the_whole_component_back() {
    let deleted = REMINDED.replace(
        "BEGIN:VEVENT\r\n\
         UID:event-1@example.com\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         SUMMARY:Weekly sync\r\n\
         BEGIN:VALARM\r\n\
         ACTION:DISPLAY\r\n\
         TRIGGER:-PT10M\r\n\
         END:VALARM\r\n\
         END:VEVENT\r\n",
        "",
    );
    let edited = REMINDED.replace("TRIGGER:-PT10M", "TRIGGER:-PT20M");

    assert!(!deleted.contains("VEVENT"), "got: {deleted}");

    for (left, right) in [(&deleted, &edited), (&edited, &deleted)] {
        let report = merge(REMINDED, left, right);
        let merged = bytes(&report);

        assert_eq!(report.conflicts.len(), 1, "{:?}", report.conflicts);
        assert_eq!(components(&merged, "VEVENT"), 1);
        assert_eq!(components(&merged, "VALARM"), 1);
        assert!(merged.contains("SUMMARY:Weekly sync"), "got: {merged}");
        assert!(merged.contains("TRIGGER:-PT20M"), "got: {merged}");
        assert_eq!(merged, reparsed(&merged));
    }
}

/// The count is the right side's blocked actions, not the properties they
/// contest: one property the left side removed and the right side both retyped
/// and relabelled is reported twice.
///
/// vcard-rs collapses the same shape to one report, so a caller showing the
/// number to a person cannot read it as a count of contested fields across
/// both libraries.
#[test]
fn a_removed_property_is_reported_once_per_edit_of_it() {
    let removed = BASE.replace("LOCATION:Room A\r\n", "");
    let touched = edited("LOCATION:Room A", "LOCATION;LANGUAGE=en:Room B");

    let report = merge(BASE, &removed, &touched);
    let merged = bytes(&report);

    assert_eq!(report.right.len(), 2);
    assert_eq!(
        collided(&report),
        ["LOCATION", "LOCATION"],
        "{:?}",
        report.conflicts
    );

    // NOTE: the property itself comes back once, the right side's own line,
    // which is what makes the two reports two views of one restoration.
    assert_eq!(merged.matches("LOCATION").count(), 1, "got: {merged}");
    assert!(
        merged.contains("LOCATION;LANGUAGE=en:Room B"),
        "got: {merged}"
    );
}
