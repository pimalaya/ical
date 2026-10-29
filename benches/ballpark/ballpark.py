"""Times python-dateutil and recurring-ical-events on the cases of
benches/recur.rs, for the ballpark table of benches/README.md.

Run by run.sh, which builds the interpreter through nix.
"""

import datetime
import pathlib
import sys
import timeit

import icalendar
import recurring_ical_events
from dateutil import rrule, tz

RULES = [
    ("daily", "FREQ=DAILY"),
    ("weekly BYDAY", "FREQ=WEEKLY;BYDAY=MO,WE,FR"),
    ("monthly BYSETPOS", "FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1"),
    ("yearly BYMONTH BYDAY", "FREQ=YEARLY;BYMONTH=3,6,9,12;BYDAY=1MO"),
    ("hourly BYHOUR", "FREQ=HOURLY;BYHOUR=9,12,17"),
]

PARIS = tz.gettz("Europe/Paris")


def report(name, seconds, runs):
    print(f"{name}\t{seconds / runs * 1e6:.1f} µs")


def first_1000(rule):
    start = datetime.datetime(2026, 1, 5, 9, tzinfo=PARIS)
    walk = rrule.rrulestr(rule, dtstart=start)
    return [at.timestamp() for _, at in zip(range(1000), walk)]


def far_window():
    start = datetime.datetime(1970, 1, 5, 9, tzinfo=PARIS)
    walk = rrule.rrulestr("FREQ=WEEKLY;BYDAY=MO,WE,FR", dtstart=start)
    return walk.between(
        datetime.datetime(2026, 7, 6, tzinfo=PARIS),
        datetime.datetime(2026, 7, 13, tzinfo=PARIS),
    )


def timeline(path):
    calendar = icalendar.Calendar.from_ical(path.read_bytes())
    return recurring_ical_events.of(calendar).between(
        datetime.date(2017, 1, 1), datetime.date(2023, 1, 1)
    )


def main():
    corpus = pathlib.Path(sys.argv[1])
    runs = 20

    for name, rule in RULES:
        report(f"dateutil: {name}", timeit.timeit(lambda: first_1000(rule), number=runs), runs)

    assert len(far_window()) == 3
    report("dateutil: far window", timeit.timeit(far_window, number=runs), runs)

    path = corpus / "rie" / "fablab_cottbus.ics"
    report("recurring-ical-events: fablab_cottbus.ics", timeit.timeit(lambda: timeline(path), number=runs), runs)


main()
