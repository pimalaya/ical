use std::{fs, hint::black_box, path::Path};

use ical::tree::cst::IcalCst;

fn walk(p: &Path, out: &mut Vec<Vec<u8>>) {
    let mut es: Vec<_> = fs::read_dir(p).unwrap().flatten().map(|e| e.path()).collect();
    es.sort();
    for e in es {
        if e.is_dir() {
            walk(&e, out)
        } else if e.extension().is_some_and(|x| x == "ics") {
            out.push(fs::read(e).unwrap())
        }
    }
}

fn main() {
    let mut a = std::env::args().skip(1);
    let work = a.next().unwrap();
    let n: usize = a.next().unwrap().parse().unwrap();
    let mut corpus = Vec::new();
    walk(Path::new("/home/soywod/code/pimalaya/ical/tests/corpus"), &mut corpus);
    if let Some(short) = work.strip_prefix("card-") {
        corpus = vec![CAL.as_bytes().to_vec(); 300];
        return run(short, n, &corpus);
    }
    run(&work, n, &corpus);
}

fn run(work: &str, n: usize, corpus: &[Vec<u8>]) {
    let csts = || {
        corpus
            .iter()
            .flat_map(|f| IcalCst::parse_many(f.as_slice()).flatten())
            .collect::<Vec<_>>()
    };
    if work == "one" {
        for _ in 0..n {
            for f in corpus {
                let _ = black_box(IcalCst::parse(black_box(f.as_slice())));
            }
        }
        return;
    }
    if work == "stats" {
        let csts = csts();
        eprintln!("files {} calendars {}", corpus.len(), csts.len());
        return;
    }
    if work == "dec" || work == "tob" || work == "str" {
        let csts = csts();
        for _ in 0..n {
            for cst in &csts {
                if work == "dec" {
                    black_box(black_box(cst).decode());
                } else if work == "str" {
                    black_box(black_box(cst).to_string());
                } else {
                    black_box(black_box(cst).to_bytes());
                }
            }
        }
        return;
    }
    if work == "encode" || work == "enc" {
        let csts = csts();
        let cals: Vec<_> = csts.iter().map(|c| c.decode()).collect();
        for _ in 0..n {
            for cal in &cals {
                if work == "encode" {
                    black_box(black_box(cal).to_string());
                } else {
                    black_box(black_box(cal).encode());
                }
            }
        }
        return;
    }
    for _ in 0..n {
        for f in corpus {
            for cal in IcalCst::parse_many(black_box(f.as_slice())) {
                let Ok(cal) = cal else { continue };
                match work {
                    "parse" => {
                        black_box(&cal);
                    }
                    "decode" => {
                        black_box(cal.decode());
                    }
                    "bytes" => {
                        black_box(cal.to_bytes());
                    }
                    _ => panic!(),
                }
            }
        }
    }
}

// NOTE: the CAL constant of benches/parse.rs, copied verbatim.
const CAL: &str = concat!(
    "BEGIN:VCALENDAR\r\n",
    "VERSION:2.0\r\n",
    "PRODID:-//Example Corp//Calendar 1.0//EN\r\n",
    "CALSCALE:GREGORIAN\r\n",
    "METHOD:REQUEST\r\n",
    "BEGIN:VEVENT\r\n",
    "UID:19970901T130000Z-123401@example.com\r\n",
    "DTSTAMP:19970901T130000Z\r\n",
    "DTSTART:19970903T163000Z\r\n",
    "DTEND:19970903T190000Z\r\n",
    "SUMMARY:Annual Employee Review\r\n",
    "DESCRIPTION:Come prepared with your goals for the year\\, and a coffee.\r\n",
    "CATEGORIES:BUSINESS,HUMAN RESOURCES\r\n",
    "ORGANIZER;CN=Jane Boss:mailto:jane@example.com\r\n",
    "ATTENDEE;ROLE=REQ-PARTICIPANT;RSVP=TRUE;CN=Joe:mailto:joe@example.com\r\n",
    "RRULE:FREQ=YEARLY;COUNT=5\r\n",
    "BEGIN:VALARM\r\n",
    "ACTION:DISPLAY\r\n",
    "DESCRIPTION:Review reminder\r\n",
    "TRIGGER:-PT1H\r\n",
    "END:VALARM\r\n",
    "END:VEVENT\r\n",
    "END:VCALENDAR\r\n",
);
