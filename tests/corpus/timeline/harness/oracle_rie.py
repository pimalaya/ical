"""The recurring-ical-events oracle.

Reads the window list (a calendar file name, a window start and a window end,
tab separated) on stdin, and writes `name<TAB>uid<TAB>start<TAB>end` for the
earliest TAKE occurrences of each series starting in the window, spelled as
the Rust side spells them. A calendar the library refuses is one
`name<TAB>ERROR<TAB>reason` line.

The one argument is tests/corpus: a calendar name with a slash is a path under
it, a bare one is under its rie folder.
"""

import signal
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

import icalendar
import recurring_ical_events

TAKE = 200
MARGIN = timedelta(days=3)
BUDGET_SECONDS = 20.0


class Timeout(Exception):
    pass


def on_alarm(signum, frame):
    raise Timeout()


def key(at):
    """A time as a naive datetime on UTC, a floating one and a date as if on
    UTC, which is how the Rust side orders and windows them."""
    if isinstance(at, datetime):
        if at.tzinfo is None:
            return at
        return at.astimezone(timezone.utc).replace(tzinfo=None)
    return datetime(at.year, at.month, at.day)


def spell(at):
    if isinstance(at, datetime):
        if at.tzinfo is None:
            return at.strftime("%Y%m%dT%H%M%S")
        return at.astimezone(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return at.strftime("%Y%m%d")


def answer(path, lo, hi):
    # NOTE: Of a file of several VCALENDARs (an iTIP exchange) the first is
    # read, as the ical-rs side reads it.
    calendar = icalendar.Calendar.from_ical(path.read_bytes(), multiple=True)[0]

    query = recurring_ical_events.of(
        calendar, components=["VEVENT", "VTODO", "VJOURNAL"]
    )
    series = {}

    for occurrence in query.occurrences_between(lo - MARGIN, hi + MARGIN):
        start = occurrence.start
        if not lo <= key(start) < hi:
            continue

        end = occurrence.end if occurrence.end is not None else start

        # NOTE: The library names a component with no UID by its Python id,
        # which changes from run to run, so every such one is labelled alike.
        uid = occurrence._adapter._component.get("UID")
        uid = "-" if uid is None else str(uid)

        series.setdefault(uid, []).append(
            (key(start), "%s\t%s" % (spell(start), spell(end)))
        )

    return series


def main():
    signal.signal(signal.SIGALRM, on_alarm)
    corpus = Path(sys.argv[1])

    for line in sys.stdin:
        name, lo, hi = line.rstrip("\n").split("\t")
        path = corpus / name if "/" in name else corpus / "rie" / name

        signal.setitimer(signal.ITIMER_REAL, BUDGET_SECONDS)
        try:
            # NOTE: A window on year 0 (a malformed calendar's) is no
            # datetime, so the calendar is refused like any other error.
            lo = datetime.strptime(lo, "%Y%m%d")
            hi = datetime.strptime(hi, "%Y%m%d")
            series = answer(path, lo, hi)
        except Timeout:
            print("%s\tERROR\tTIMEOUT" % name)
            continue
        except Exception as error:
            print("%s\tERROR\t%s" % (name, type(error).__name__))
            continue
        finally:
            signal.setitimer(signal.ITIMER_REAL, 0)

        for uid in sorted(series):
            for _, row in sorted(series[uid])[:TAKE]:
                print("%s\t%s\t%s" % (name, uid, row))


if __name__ == "__main__":
    main()
