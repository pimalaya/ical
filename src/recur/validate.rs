//! # Rule validation
//!
//! The "strict out" check for a recurrence rule (RFC 5545 3.3.10).
//!
//! Expansion is liberal by design: a `BY` part the frequency forbids is
//! ignored rather than refused, because that is what "liberal in what it
//! accepts" means for a rule that arrived from someone else's server.
//!
//! That leaves a caller who is *writing* a rule with no way to learn it is
//! malformed, which is what this is for. The two never disagree: validation
//! reports the part, expansion still ignores it.
//!
//! A rule that passes earns an [`IcalValid`], the same proof
//! [`Ical::validate`](crate::ical::Ical::validate) mints for a whole calendar.

use core::{error, fmt};

use alloc::vec::Vec;

use crate::{
    recur::{IcalRecurFreq, IcalRecurRule, IcalRecurSkip},
    validator::IcalValid,
};

/// A rule part, named so a problem can point at one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalRecurPart {
    /// `BYSECOND`.
    BySecond,
    /// `BYMINUTE`.
    ByMinute,
    /// `BYHOUR`.
    ByHour,
    /// `BYDAY`.
    ByDay,
    /// `BYMONTHDAY`.
    ByMonthDay,
    /// `BYYEARDAY`.
    ByYearDay,
    /// `BYWEEKNO`.
    ByWeekNo,
    /// `BYMONTH`.
    ByMonth,
    /// `BYSETPOS`.
    BySetPos,
}

impl fmt::Display for IcalRecurPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BySecond => "BYSECOND",
            Self::ByMinute => "BYMINUTE",
            Self::ByHour => "BYHOUR",
            Self::ByDay => "BYDAY",
            Self::ByMonthDay => "BYMONTHDAY",
            Self::ByYearDay => "BYYEARDAY",
            Self::ByWeekNo => "BYWEEKNO",
            Self::ByMonth => "BYMONTH",
            Self::BySetPos => "BYSETPOS",
        })
    }
}

/// One way a rule breaks RFC 5545 3.3.10.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalRecurRuleProblem {
    /// A `BY` part the rule's frequency forbids.
    PartFreq {
        /// The part that may not appear.
        part: IcalRecurPart,
        /// The frequency that forbids it.
        freq: IcalRecurFreq,
    },
    /// A `BYDAY` ordinal (`2MO`, `-1SU`) outside `MONTHLY` and `YEARLY`.
    OrdinalFreq {
        /// The frequency that forbids it.
        freq: IcalRecurFreq,
    },
    /// A `BYDAY` ordinal at `YEARLY` beside a `BYWEEKNO`, where it would mean
    /// two contradictory things at once.
    OrdinalWithWeekNo,
    /// `BYSETPOS` with no other `BY` part to pick positions out of.
    SetPosAlone,
    /// `UNTIL` and `COUNT` together, which bound the rule twice.
    UntilWithCount,
    /// `SKIP` with no `RSCALE`, which RFC 7529 4 requires beside it.
    SkipWithoutScale,
}

impl fmt::Display for IcalRecurRuleProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PartFreq { part, freq } => {
                write!(f, "{part} may not be used with FREQ={}", freq_name(*freq))
            }
            Self::OrdinalFreq { freq } => {
                write!(
                    f,
                    "a BYDAY ordinal may not be used with FREQ={}",
                    freq_name(*freq)
                )
            }
            Self::OrdinalWithWeekNo => {
                f.write_str("a BYDAY ordinal may not be used with FREQ=YEARLY beside BYWEEKNO")
            }
            Self::SetPosAlone => f.write_str("BYSETPOS needs another BY part to pick from"),
            Self::UntilWithCount => f.write_str("UNTIL and COUNT may not both be given"),
            Self::SkipWithoutScale => f.write_str("SKIP may not be used without RSCALE"),
        }
    }
}

impl error::Error for IcalRecurRuleProblem {}

/// The wire spelling of a frequency.
fn freq_name(freq: IcalRecurFreq) -> &'static str {
    match freq {
        IcalRecurFreq::Secondly => "SECONDLY",
        IcalRecurFreq::Minutely => "MINUTELY",
        IcalRecurFreq::Hourly => "HOURLY",
        IcalRecurFreq::Daily => "DAILY",
        IcalRecurFreq::Weekly => "WEEKLY",
        IcalRecurFreq::Monthly => "MONTHLY",
        IcalRecurFreq::Yearly => "YEARLY",
    }
}

impl IcalRecurRule {
    /// Check the rule against RFC 5545 3.3.10, returning an [`IcalValid`] proof
    /// or every problem found.
    pub fn validate(self) -> Result<IcalValid<Self>, Vec<IcalRecurRuleProblem>> {
        let problems = self.problems();

        if problems.is_empty() {
            Ok(IcalValid(self))
        } else {
            Err(problems)
        }
    }

    /// Every RFC 5545 3.3.10 constraint this rule breaks, in the order the
    /// section states them. Empty for a conformant rule.
    pub fn problems(&self) -> Vec<IcalRecurRuleProblem> {
        use IcalRecurFreq::*;
        use IcalRecurPart::*;

        let mut problems = Vec::new();
        let freq = self.freq;

        // NOTE: "The BYWEEKNO rule part MUST NOT be used when the FREQ rule
        // part is set to anything other than YEARLY."
        if !self.by_week_no.is_empty() && freq != Yearly {
            problems.push(IcalRecurRuleProblem::PartFreq {
                part: ByWeekNo,
                freq,
            });
        }

        // NOTE: "The BYYEARDAY rule part MUST NOT be specified when the FREQ
        // rule part is set to DAILY, WEEKLY, or MONTHLY."
        if !self.by_year_day.is_empty() && matches!(freq, Daily | Weekly | Monthly) {
            problems.push(IcalRecurRuleProblem::PartFreq {
                part: ByYearDay,
                freq,
            });
        }

        // NOTE: "The BYMONTHDAY rule part MUST NOT be specified when the FREQ
        // rule part is set to WEEKLY."
        if !self.by_month_day.is_empty() && freq == Weekly {
            problems.push(IcalRecurRuleProblem::PartFreq {
                part: ByMonthDay,
                freq,
            });
        }

        // NOTE: "The BYDAY rule part MUST NOT be specified with a numeric value
        // when the FREQ rule part is not set to MONTHLY or YEARLY. Furthermore,
        // the BYDAY rule part MUST NOT be specified with a numeric value with
        // the FREQ rule part set to YEARLY when the BYWEEKNO rule part is
        // specified."
        let ordinal = self.by_day.iter().any(|day| day.ordinal.is_some());
        if ordinal {
            if !matches!(freq, Monthly | Yearly) {
                problems.push(IcalRecurRuleProblem::OrdinalFreq { freq });
            } else if freq == Yearly && !self.by_week_no.is_empty() {
                problems.push(IcalRecurRuleProblem::OrdinalWithWeekNo);
            }
        }

        // NOTE: "[BYSETPOS] MUST only be used in conjunction with another BYxxx
        // rule part."
        if !self.by_set_pos.is_empty() && !self.has_other_by_part() {
            problems.push(IcalRecurRuleProblem::SetPosAlone);
        }

        // NOTE: "The UNTIL rule part and the COUNT rule part MUST NOT occur in
        // the same 'recur'." Parsing accepts both, since a rule that says too
        // much is still a rule; this is where it is said out loud.
        if self.until.is_some() && self.count.is_some() {
            problems.push(IcalRecurRuleProblem::UntilWithCount);
        }

        // NOTE: RFC 7529 4: SKIP "MUST NOT be present unless RSCALE is
        // present". `RSCALE=GREGORIAN` is the usual way to satisfy that, and
        // the only one this crate expands.
        if self.skip != IcalRecurSkip::Omit && self.scale.is_none() {
            problems.push(IcalRecurRuleProblem::SkipWithoutScale);
        }

        problems
    }

    /// Whether the rule carries a `BY` part other than `BYSETPOS`.
    fn has_other_by_part(&self) -> bool {
        !self.by_second.is_empty()
            || !self.by_minute.is_empty()
            || !self.by_hour.is_empty()
            || !self.by_day.is_empty()
            || !self.by_month_day.is_empty()
            || !self.by_year_day.is_empty()
            || !self.by_week_no.is_empty()
            || !self.by_month.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::recur::{
        IcalRecurDateTime, IcalRecurFreq, IcalRecurRule,
        validate::{IcalRecurPart, IcalRecurRuleProblem},
    };

    fn problems(rule: &str) -> vec::Vec<IcalRecurRuleProblem> {
        IcalRecurRule::parse(rule)
            .expect("a readable rule")
            .problems()
    }

    #[test]
    fn accepts_the_rules_the_rfc_writes() {
        for rule in [
            "FREQ=YEARLY;BYWEEKNO=20;BYDAY=MO",
            "FREQ=MONTHLY;BYDAY=2MO",
            "FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10",
            "FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1",
            "FREQ=SECONDLY;BYYEARDAY=1",
            "FREQ=DAILY;COUNT=10",
        ] {
            assert!(problems(rule).is_empty(), "{rule} should validate");
        }
    }

    #[test]
    fn reports_a_part_the_frequency_forbids() {
        assert_eq!(
            problems("FREQ=MONTHLY;BYWEEKNO=3"),
            [IcalRecurRuleProblem::PartFreq {
                part: IcalRecurPart::ByWeekNo,
                freq: IcalRecurFreq::Monthly,
            }]
        );
        assert_eq!(
            problems("FREQ=WEEKLY;BYYEARDAY=100"),
            [IcalRecurRuleProblem::PartFreq {
                part: IcalRecurPart::ByYearDay,
                freq: IcalRecurFreq::Weekly,
            }]
        );
        assert_eq!(
            problems("FREQ=WEEKLY;BYMONTHDAY=15"),
            [IcalRecurRuleProblem::PartFreq {
                part: IcalRecurPart::ByMonthDay,
                freq: IcalRecurFreq::Weekly,
            }]
        );
    }

    #[test]
    fn reports_an_ordinal_where_it_means_nothing() {
        assert_eq!(
            problems("FREQ=WEEKLY;BYDAY=2MO"),
            [IcalRecurRuleProblem::OrdinalFreq {
                freq: IcalRecurFreq::Weekly,
            }]
        );
        assert_eq!(
            problems("FREQ=YEARLY;BYWEEKNO=20;BYDAY=2MO"),
            [IcalRecurRuleProblem::OrdinalWithWeekNo]
        );
    }

    #[test]
    fn reports_a_setpos_with_nothing_to_pick_from() {
        assert_eq!(
            problems("FREQ=DAILY;BYSETPOS=2"),
            [IcalRecurRuleProblem::SetPosAlone]
        );
        assert!(problems("FREQ=DAILY;BYHOUR=9,17;BYSETPOS=2").is_empty());
    }

    #[test]
    fn reports_a_rule_bounded_twice() {
        // NOTE: Parsing refuses this pair, so the case has to be built by hand,
        // which is exactly the caller this check is for.
        let mut rule = IcalRecurRule::parse("FREQ=DAILY;COUNT=3").unwrap();
        rule.until = Some(IcalRecurDateTime::date(2026, 1, 1));

        assert_eq!(rule.problems(), [IcalRecurRuleProblem::UntilWithCount]);
    }

    #[test]
    fn a_valid_rule_mints_a_proof() {
        let rule = IcalRecurRule::parse("FREQ=MONTHLY;BYDAY=2MO").unwrap();
        let valid = rule.validate().expect("a conformant rule");

        assert_eq!(valid.freq, IcalRecurFreq::Monthly);
    }

    #[test]
    fn expansion_stays_liberal_about_what_validation_reports() {
        use crate::recur::{IcalRecurDateTime, expand::IcalRecurExpand};

        let rule = IcalRecurRule::parse("FREQ=MONTHLY;BYWEEKNO=3").unwrap();
        assert!(!rule.problems().is_empty());

        let start = IcalRecurDateTime::date(2026, 1, 15);
        let occurrences: vec::Vec<_> = IcalRecurExpand::new(rule, start).take(2).collect();

        assert_eq!(occurrences, [start, IcalRecurDateTime::date(2026, 2, 15)]);
    }
}
