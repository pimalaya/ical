//! Resolve the zones a calendar names but does not define, through the bundled
//! IANA database.
//!
//! RFC 5545 makes a calendar carry a `VTIMEZONE` for every `TZID` it uses, and
//! many do not. Behind the `tzdb` feature, the zones a calendar's own
//! definitions leave out are completed from the database jiff bundles, the
//! calendar's definition always winning. A Windows name, as Outlook writes it,
//! reads as the IANA zone CLDR maps it to.
//!
//! The same database makes the `X-WR-TIMEZONE` extension usable: a calendar
//! written in UTC that names its zone this way is read on that zone, as
//! calendar applications read it, so a weekly event stays at the same local
//! hour across a transition.
//!
//! Run with: `cargo run --example time_zone_database --features tzdb`

use ical::{
    recur::{IcalRecurDateTime, instant::IcalRecurTimeline},
    tree::cst::IcalCst,
    tz::IcalTz,
    tzdb,
};

fn main() {
    println!(
        "bundled IANA release: {}",
        tzdb::version().unwrap_or("unknown")
    );

    // A meeting under a `TZID` the calendar defines no `VTIMEZONE` for.
    let raw = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:sync@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART;TZID=America/New_York:20260302T100000\r\n",
        "DURATION:PT30M\r\n",
        "RRULE:FREQ=WEEKLY;COUNT=3\r\n",
        "SUMMARY:Sync\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );
    let cst = IcalCst::parse(raw).unwrap();
    let cal = cst.decode();

    // The calendar's own zones alone know nothing of New York, so its times
    // would read floating. Completed through 2030 from the database, the walk
    // follows the transition of 2026-03-08: 10:00 local is 15:00 UTC, then
    // 14:00.
    let zones = tzdb::zones(&cal.components, 2030);
    let timeline = IcalRecurTimeline::of_uid(&cal.components, "sync@example.com", &zones);

    println!("\nthe sync at 10:00 in New York, UTC:");
    for instant in timeline.expand(i64::MIN, i64::MAX) {
        println!("  {}", utc(instant.start));
    }

    // A Windows name, as Outlook writes it, resolves through CLDR to
    // America/Los_Angeles: UTC-0700 in July.
    let pacific = tzdb::zone("Pacific Standard Time", 2030).expect("a CLDR name");
    let july = IcalRecurDateTime::date(2026, 7, 1);
    let offset = pacific.resolve(july).unambiguous().unwrap_or_default();
    println!(
        "\n`Pacific Standard Time` in July: UTC{:+03}00",
        offset / 3600
    );

    // A calendar in UTC naming its zone with `X-WR-TIMEZONE`. Read as written,
    // the event stays at 08:00 UTC and so moves an hour in Paris once the
    // clocks change on 2026-03-29.
    let raw = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "X-WR-TIMEZONE:Europe/Paris\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:class@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260320T080000Z\r\n",
        "RRULE:FREQ=WEEKLY;COUNT=3\r\n",
        "SUMMARY:Class\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );
    let cst = IcalCst::parse(raw).unwrap();
    let mut cal = cst.decode();

    let name = IcalTz::x_wr_timezone(&cal)
        .expect("the zone the calendar names")
        .to_owned();
    let zones: Vec<IcalTz> = tzdb::zone(&name, 2030).into_iter().collect();

    let walk = |cal: &ical::ical::Ical<'_>| {
        let timeline = IcalRecurTimeline::of_uid(&cal.components, "class@example.com", &zones);
        timeline
            .expand(i64::MIN, i64::MAX)
            .map(|instant| utc(instant.start))
            .collect::<Vec<_>>()
            .join(", ")
    };

    println!("\nthe class as written, UTC:     {}", walk(&cal));

    // Told on Paris, the rule expands on its wall clock: 09:00 local every
    // week, which is 07:00 UTC once summer time starts.
    IcalTz::apply_x_wr_timezone(&mut cal, &zones);
    println!("the class on {name}, UTC: {}", walk(&cal));
}

/// An instant in seconds since the Unix epoch, as a UTC date-time.
fn utc(seconds: i64) -> String {
    let at = IcalRecurDateTime::from_seconds(seconds);

    format!(
        "{}-{:02}-{:02} {:02}:{:02}",
        at.year, at.month, at.day, at.hour, at.minute,
    )
}
