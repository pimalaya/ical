"""The python-dateutil oracle.

Reads the case list (a start and a rule per line, tab separated) on stdin and
writes `start<TAB>rule<TAB>occurrences` on stdout, twelve occurrences at most.

Every case runs under its own alarm: a rule dateutil cannot satisfy walks to
year 9999 one tick at a time, and the iterator is pure Python, so the signal
handler lands between bytecodes and unwinds the case rather than the run.
"""

import signal
import sys
from datetime import datetime

from dateutil.rrule import rrulestr

TAKE = 12
BUDGET_SECONDS = 2.0


class Timeout(Exception):
    pass


def on_alarm(signum, frame):
    raise Timeout()


def occurrences(start, rule):
    parsed = rrulestr("DTSTART:%s\nRRULE:%s" % (start, rule))
    out = []
    for index, moment in enumerate(parsed):
        if index >= TAKE:
            break
        out.append(moment.strftime("%Y%m%dT%H%M%S"))
    return ",".join(out)


def main():
    signal.signal(signal.SIGALRM, on_alarm)

    for line in sys.stdin:
        line = line.rstrip("\n")
        if not line or "\t" not in line:
            continue

        start, rule = line.split("\t", 1)

        try:
            datetime.strptime(start, "%Y%m%dT%H%M%S")
        except ValueError:
            print("%s\t%s\tSTART_ERROR" % (start, rule))
            continue

        signal.setitimer(signal.ITIMER_REAL, BUDGET_SECONDS)
        try:
            answer = occurrences(start, rule)
        except Timeout:
            answer = "TIMEOUT"
        except Exception:
            answer = "ERROR"
        finally:
            signal.setitimer(signal.ITIMER_REAL, 0)

        print("%s\t%s\t%s" % (start, rule, answer))


if __name__ == "__main__":
    main()
