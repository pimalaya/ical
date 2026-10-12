//! Nesting bounds, over calendars built to exhaust the stack.
//!
//! Every walk of the tree recurses once per component level, so the parsers
//! bound the depth they build. Each case runs on a 1 MiB thread, the stack an
//! ordinary Android thread gets, where an unbounded parse overflowed at a few
//! hundred levels: about 9 KB of `BEGIN` lines.

#![cfg(feature = "parser")]

use std::thread;

use ical::tree::{
    cst::{IcalCst, IcalItem},
    error::IcalParseError,
    merge::IcalMerge,
};

/// A calendar nesting `depth` components, the root included.
fn nested(depth: usize) -> String {
    let mut out = String::from("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n");

    for _ in 1..depth {
        out.push_str("BEGIN:VEVENT\r\nUID:x\r\n");
    }
    for _ in 1..depth {
        out.push_str("END:VEVENT\r\n");
    }

    out.push_str("END:VCALENDAR\r\n");
    out
}

/// How deep a tree nests, the root counting as one.
fn depth(cst: &IcalCst<'_>) -> usize {
    1 + cst
        .items
        .iter()
        .filter_map(|item| match item {
            IcalItem::Component(child) => Some(depth(child)),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

/// Run a case on a 1 MiB stack.
fn on_small_stack(case: impl FnOnce() + Send + 'static) {
    thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(case)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn strict_parsers_refuse_nesting_past_the_bound() {
    on_small_stack(|| {
        let input = nested(10_000);

        let error = IcalCst::parse(&input).unwrap_err();
        assert_eq!(error, IcalParseError::TooDeep("VEVENT".into()));

        let first = IcalCst::parse_many(&input).next().unwrap();
        assert!(matches!(first, Err(IcalParseError::TooDeep(_))));
    });
}

#[test]
fn strict_parser_accepts_nesting_at_the_bound() {
    on_small_stack(|| {
        let input = nested(IcalCst::DEPTH_MAX);
        let cst = IcalCst::parse(&input).unwrap();

        assert_eq!(depth(&cst), IcalCst::DEPTH_MAX);
        assert_eq!(cst.to_bytes(), input.as_bytes());

        let input = nested(IcalCst::DEPTH_MAX + 1);
        assert!(IcalCst::parse(&input).is_err());
    });
}

#[test]
fn recovering_parser_keeps_deeper_lines_raw() {
    on_small_stack(|| {
        let input = nested(2_000);
        let recovery = IcalCst::parse_recovering(&input);

        assert_eq!(recovery.to_bytes(), input.as_bytes());
        assert_eq!(
            recovery.problems,
            [IcalParseError::TooDeep("VEVENT".into())]
        );

        let cst = &recovery.calendars[0];
        assert_eq!(depth(cst), IcalCst::DEPTH_MAX);

        // NOTE: Everything walking the tree stays within the stack too.
        let _ = cst.decode();
        let _ = cst.clone();
        let report = IcalMerge {
            base: cst,
            left: cst,
            right: cst,
        }
        .merge();
        assert!(report.conflicts.is_empty());
    });
}

#[test]
fn recovering_parser_keeps_an_unclosed_deep_tail() {
    on_small_stack(|| {
        let input: String =
            nested(10_000)
                .lines()
                .take(15_000)
                .fold(String::new(), |mut out, line| {
                    out.push_str(line);
                    out.push_str("\r\n");
                    out
                });
        let recovery = IcalCst::parse_recovering(&input);

        assert_eq!(recovery.to_bytes(), input.as_bytes());
        assert!(
            recovery
                .problems
                .contains(&IcalParseError::TooDeep("VEVENT".into()))
        );
        assert!(matches!(
            recovery.problems.last(),
            Some(IcalParseError::MissingEnd(_))
        ));
    });
}
