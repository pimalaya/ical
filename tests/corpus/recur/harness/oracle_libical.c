/* The libical oracle.
 *
 * Reads the case list (a start and a rule per line, tab separated) on stdin and
 * writes `start<TAB>rule<TAB>occurrences` on stdout, twelve occurrences at most.
 *
 * Every case runs under its own alarm, since a rule libical cannot satisfy
 * walks to year 9999 one tick at a time. The handler jumps out of the iterator,
 * which leaks the iterator: acceptable in a one-shot generator, and the reason
 * every buffer here is static rather than automatic (an automatic non-volatile
 * object modified between sigsetjmp and siglongjmp is indeterminate).
 *
 * Build: cc -o libical-oracle libical.c $(pkg-config --cflags --libs libical)
 */

#include <setjmp.h>
#include <signal.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include <libical/ical.h>

#define TAKE 12
#define BUDGET_SECONDS 2

static sigjmp_buf escape;
static char line[8192];
static char answer[8192];

static void on_alarm(int signum)
{
    (void)signum;
    siglongjmp(escape, 1);
}

static void expand(const char *start, const char *rule)
{
    struct icalrecurrencetype recur;
    struct icaltimetype dtstart;
    icalrecur_iterator *iterator;
    int index;

    recur = icalrecurrencetype_from_string(rule);
    if (recur.freq == ICAL_NO_RECURRENCE) {
        strcpy(answer, "REFUSED");
        return;
    }

    dtstart = icaltime_from_string(start);
    if (icaltime_is_null_time(dtstart)) {
        strcpy(answer, "START_ERROR");
        return;
    }

    iterator = icalrecur_iterator_new(recur, dtstart);
    if (iterator == NULL) {
        strcpy(answer, "REFUSED");
        return;
    }

    for (index = 0; index < TAKE; index++) {
        struct icaltimetype next = icalrecur_iterator_next(iterator);
        char moment[32];

        if (icaltime_is_null_time(next)) {
            break;
        }

        snprintf(moment, sizeof moment, "%04d%02d%02dT%02d%02d%02d", next.year,
                 next.month, next.day, next.hour, next.minute, next.second);

        if (answer[0] != '\0') {
            strcat(answer, ",");
        }
        strcat(answer, moment);
    }

    icalrecur_iterator_free(iterator);
}

int main(void)
{
    /* A malformed rule must answer REFUSED, not kill the run. */
    icalerror_set_errors_are_fatal(0);
    signal(SIGALRM, on_alarm);

    while (fgets(line, sizeof line, stdin) != NULL) {
        char *tab;
        const char *start;
        const char *rule;

        line[strcspn(line, "\n")] = '\0';
        tab = strchr(line, '\t');
        if (tab == NULL) {
            continue;
        }

        *tab = '\0';
        start = line;
        rule = tab + 1;
        answer[0] = '\0';

        if (sigsetjmp(escape, 1) == 0) {
            alarm(BUDGET_SECONDS);
            expand(start, rule);
            alarm(0);
        } else {
            alarm(0);
            strcpy(answer, "TIMEOUT");
        }

        printf("%s\t%s\t%s\n", start, rule, answer);
    }

    return 0;
}
