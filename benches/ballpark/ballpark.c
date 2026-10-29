/*
 * Times libical on the rule cases of benches/recur.rs, for the ballpark
 * table of benches/README.md. Built and run by run.sh.
 */

#include <libical/ical.h>
#include <stdio.h>
#include <time.h>

static const char *RULES[][2] = {
    {"daily", "FREQ=DAILY"},
    {"weekly BYDAY", "FREQ=WEEKLY;BYDAY=MO,WE,FR"},
    {"monthly BYSETPOS", "FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1"},
    {"yearly BYMONTH BYDAY", "FREQ=YEARLY;BYMONTH=3,6,9,12;BYDAY=1MO"},
    {"hourly BYHOUR", "FREQ=HOURLY;BYHOUR=9,12,17"},
};

static double now(void) {
    struct timespec at;
    clock_gettime(CLOCK_MONOTONIC, &at);
    return at.tv_sec + at.tv_nsec / 1e9;
}

/* The first `take` occurrences as instants, each resolved through the
 * builtin Europe/Paris zone, as the Rust side resolves its own. */
static long first(const char *rule, const char *start, int take) {
    icaltimezone *paris = icaltimezone_get_builtin_timezone("Europe/Paris");
    struct icalrecurrencetype recur = icalrecurrencetype_from_string(rule);
    struct icaltimetype dtstart = icaltime_from_string(start);
    dtstart = icaltime_set_timezone(&dtstart, paris);

    icalrecur_iterator *walk = icalrecur_iterator_new(recur, dtstart);
    long sum = 0;

    for (int i = 0; i < take; i++) {
        struct icaltimetype at = icalrecur_iterator_next(walk);
        if (icaltime_is_null_time(at))
            break;
        sum += icaltime_as_timet_with_zone(at, paris);
    }

    icalrecur_iterator_free(walk);
    return sum;
}

int main(void) {
    int runs = 200;

    for (size_t i = 0; i < sizeof RULES / sizeof *RULES; i++) {
        double started = now();
        for (int run = 0; run < runs; run++)
            first(RULES[i][1], "20260105T090000", 1000);
        printf("libical: %s\t%.1f µs\n", RULES[i][0], (now() - started) / runs * 1e6);
    }

    return 0;
}
