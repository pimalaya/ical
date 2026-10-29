---
cairn: log
change: non-gregorian-scales
date: 2026-10-29
---

# Non-Gregorian scales

- `icu_calendar` 2.3.0 chosen: no_std, MSRV 1.88, about 7 KB, Chinese tables 1900 to 2100. Siblings released 2026-10-22 pinned back in the lock; deny.toml allows Unicode-3.0.
- **Breaking**: `by_month` is `Vec<IcalRecurMonth>` (1 to 13, leap flag); `IcalRecurRuleProblem::MonthScale` flags a 13th or leap month on the Gregorian scale.
- Feature `rscale`: src/recur/scale.rs feeds the expander every year, month and day question; leap months `SKIP` per RFC 7529 4.1. 15 scales; `ISLAMIC` and `ISLAMIC-RGSA` yield nothing.
- RFC 7529 4.3 examples pass verbatim. rscale.sh against libical with ICU 76: 1,912 cases agree (rscale.tsv), 458 differ in ten classes pinned in rscale-divergence.tsv; nine are libical bugs, one (Chinese past 2100) is unknown.

Capabilities moved: **recurrence**, **conformance**.
