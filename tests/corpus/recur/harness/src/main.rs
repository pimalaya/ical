//! Regeneration tool for the frozen recurrence corpus.
//!
//! Three subcommands, chained by `run.sh`:
//!
//! - `generate` writes the case list (a start and a rule per line) to stdout.
//! - `expand` reads that case list and answers it through ical-rs.
//! - `cross` reads the three answer files (ical-rs, dateutil, libical) and
//!   writes the consensus corpus: every case the two oracles answer alike.
//!
//! The oracles are python-dateutil (`dateutil.py`) and libical (`libical.c`),
//! each with its own per-case alarm, since a rule neither can satisfy walks to
//! year 9999 one tick at a time.
//!
//! Nothing here ships with the crate: `tests/corpus/recur/harness` is a
//! detached package that cargo never builds as part of ical-rs.

use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write, stdin, stdout},
    process::exit,
};

use ical::recur::{IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand};

/// How many occurrences each case asks for.
const TAKE: usize = 12;

/// The frequencies every sweep crosses.
const FREQS: [&str; 7] = [
    "SECONDLY",
    "MINUTELY",
    "HOURLY",
    "DAILY",
    "WEEKLY",
    "MONTHLY",
    "YEARLY",
];

/// One `BY` part (or modifier) per entry, each with a value chosen for its
/// edges: a single value, a list, a negative index, an impossible date.
const PARTS: [&str; 34] = [
    "BYSECOND=0,30",
    "BYSECOND=15",
    "BYMINUTE=0,30",
    "BYMINUTE=5",
    "BYHOUR=9",
    "BYHOUR=0,12,23",
    "BYDAY=MO",
    "BYDAY=MO,WE,FR",
    "BYDAY=2MO",
    "BYDAY=-1SU",
    "BYDAY=SU",
    "BYMONTHDAY=1",
    "BYMONTHDAY=15",
    "BYMONTHDAY=-1",
    "BYMONTHDAY=29,31",
    "BYYEARDAY=1",
    "BYYEARDAY=-1",
    "BYYEARDAY=100,200",
    "BYWEEKNO=1",
    "BYWEEKNO=20",
    "BYWEEKNO=-1",
    "BYWEEKNO=53",
    "BYMONTH=1",
    "BYMONTH=2",
    "BYMONTH=6",
    "BYMONTH=1,7",
    "BYSETPOS=1",
    "BYSETPOS=-1",
    "BYSETPOS=2",
    "BYSETPOS=1,-1",
    "INTERVAL=2",
    "INTERVAL=7",
    "WKST=SU",
    "WKST=MO",
];

/// Part pairs that interact: an ordinal narrowed by another part, a limit
/// crossed with an expansion, a set `BYSETPOS` then picks from.
const PAIRS: [&str; 12] = [
    "BYMONTH=6;BYDAY=SU",
    "BYMONTHDAY=15;BYDAY=2MO",
    "BYWEEKNO=20;BYDAY=MO",
    "BYYEARDAY=100;BYDAY=FR",
    "BYDAY=MO,TU;BYSETPOS=-1",
    "BYMONTHDAY=1,-1;BYSETPOS=1",
    "BYHOUR=9,17;BYMINUTE=0,30",
    "BYMONTH=2;BYMONTHDAY=29",
    "BYDAY=-1SU;BYMONTH=10",
    "INTERVAL=3;BYMONTHDAY=-1",
    "BYSECOND=0;BYMINUTE=0;BYHOUR=0",
    "WKST=SU;BYDAY=SU;BYWEEKNO=1",
];

/// The `UNTIL` and `COUNT` bounds. `UNTIL` stays naive: the starts are floating,
/// and a UTC `UNTIL` against a floating start is a case dateutil refuses.
const BOUNDS: [&str; 5] = [
    "COUNT=1",
    "COUNT=3",
    "COUNT=25",
    "UNTIL=20270101T000000",
    "UNTIL=20260601T120000",
];

/// Composite rules of the shape real calendars carry.
const COMPOSITES: [&str; 12] = [
    "FREQ=YEARLY;INTERVAL=2;BYMONTH=1;BYDAY=SU;BYHOUR=8,9;BYMINUTE=30",
    "FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1",
    "FREQ=MONTHLY;BYDAY=FR;BYMONTHDAY=13",
    "FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10",
    "FREQ=YEARLY;BYDAY=2SU;BYMONTH=3",
    "FREQ=WEEKLY;INTERVAL=2;WKST=SU;BYDAY=TU,TH",
    "FREQ=MONTHLY;INTERVAL=18;BYMONTHDAY=10,11,12,13,14,15",
    "FREQ=DAILY;BYHOUR=9,12,15;BYMINUTE=0,20,40",
    "FREQ=YEARLY;BYWEEKNO=20;BYDAY=MO",
    "FREQ=MINUTELY;INTERVAL=15;BYHOUR=9,10",
    "FREQ=MONTHLY;INTERVAL=7;BYMONTH=6;BYDAY=SU;BYMONTHDAY=1",
    "FREQ=SECONDLY;BYSETPOS=2",
];

/// The eight starts, chosen for their edges: a leap day, two month ends, a year
/// end, three Mondays, and one ordinary afternoon.
const STARTS: [&str; 8] = [
    "20240229T090000",
    "20260131T235959",
    "20251231T120000",
    "20260105T090000",
    "20260302T000000",
    "20260701T083000",
    "20260930T120000",
    "20260214T140000",
];

fn main() {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("generate") => generate(),
        Some("expand") => expand(),
        Some("cross") => {
            let files: Vec<String> = args.collect();
            if files.len() != 3 {
                eprintln!("usage: harness cross <ical.tsv> <dateutil.tsv> <libical.tsv>");
                exit(2);
            }
            cross(&files[0], &files[1], &files[2]);
        }
        _ => {
            eprintln!("usage: harness generate | expand | cross <a> <b> <c>");
            exit(2);
        }
    }
}

/// Writes every case, one `start<TAB>rule` per line.
fn generate() {
    let out = stdout();
    let mut out = BufWriter::new(out.lock());
    let mut emit = |start: &str, rule: &str| {
        writeln!(out, "{start}\t{rule}").expect("write case");
    };

    // Every frequency crossed with every part, from every start.
    for freq in FREQS {
        for part in PARTS {
            for start in STARTS {
                emit(start, &format!("FREQ={freq};{part}"));
            }
        }
    }

    // The interacting pairs, likewise.
    for freq in FREQS {
        for pair in PAIRS {
            for start in STARTS {
                emit(start, &format!("FREQ={freq};{pair}"));
            }
        }
    }

    // The bounds.
    for freq in FREQS {
        for bound in BOUNDS {
            for start in STARTS {
                emit(start, &format!("FREQ={freq};{bound}"));
            }
        }
    }

    // The composites.
    for rule in COMPOSITES {
        for start in STARTS {
            emit(start, rule);
        }
    }

    // Six thousand seeded random combinations of two to four parts. The seed is
    // fixed, so the case list is reproducible without committing it separately.
    let mut rng = Lcg::new(0x5445_5354_5245_4355);
    for _ in 0..6000 {
        let freq = FREQS[rng.below(FREQS.len())];
        let start = STARTS[rng.below(STARTS.len())];
        let count = 2 + rng.below(3);

        let mut rule = format!("FREQ={freq}");
        let mut used: Vec<&str> = Vec::new();

        for _ in 0..count {
            let part = PARTS[rng.below(PARTS.len())];
            let name = part.split('=').next().unwrap();
            if used.contains(&name) {
                continue;
            }
            used.push(name);
            rule.push(';');
            rule.push_str(part);
        }

        emit(start, &rule);
    }
}

/// Answers every case on stdin through ical-rs.
fn expand() {
    let input = stdin();
    let out = stdout();
    let mut out = BufWriter::new(out.lock());

    for line in input.lock().lines() {
        let line = line.expect("read case");
        let Some((start, rule)) = line.split_once('\t') else {
            continue;
        };

        writeln!(out, "{start}\t{rule}\t{}", answer(start, rule)).expect("write answer");
    }
}

/// The comma-joined occurrences of one case, or a marker for a case ical-rs
/// refuses.
fn answer(start: &str, rule: &str) -> String {
    let Ok(start) = IcalRecurDateTime::parse(start) else {
        return String::from("START_ERROR");
    };

    let Ok(rule) = IcalRecurRule::parse(rule) else {
        return String::from("PARSE_ERROR");
    };

    let occurrences: Vec<String> = IcalRecurExpand::new(rule, start)
        .take(TAKE)
        .map(|occurrence| {
            format!(
                "{:04}{:02}{:02}T{:02}{:02}{:02}",
                occurrence.year,
                occurrence.month,
                occurrence.day,
                occurrence.hour,
                occurrence.minute,
                occurrence.second,
            )
        })
        .collect();

    occurrences.join(",")
}

/// Crosses the three answer files and writes the consensus corpus to stdout,
/// reporting the counts and every case where ical-rs is the lone dissenter.
fn cross(ical: &str, dateutil: &str, libical: &str) {
    let ical = read_answers(ical);
    let dateutil = read_answers(dateutil);
    let libical = read_answers(libical);

    let out = stdout();
    let mut out = BufWriter::new(out.lock());

    let mut agreed = 0usize;
    let mut matched = 0usize;
    let mut dissent: Vec<(String, String, String, String)> = Vec::new();

    // The case order is the generator's, so iterate the ical-rs answers.
    for (key, ours) in &ical.order {
        let (Some(left), Some(right)) = (dateutil.by_key.get(key), libical.by_key.get(key)) else {
            continue;
        };

        // An oracle that errored, timed out or refused the rule is not an
        // opinion, so the case cannot be a consensus.
        if is_marker(left) || is_marker(right) || left != right {
            continue;
        }

        agreed += 1;

        let (start, rule) = key.split_once('\t').expect("keyed case");
        writeln!(out, "{start}\t{rule}\t{left}").expect("write consensus");

        if ours == left {
            matched += 1;
        } else {
            dissent.push((
                start.to_string(),
                rule.to_string(),
                left.clone(),
                ours.clone(),
            ));
        }
    }

    eprintln!("cases:      {}", ical.order.len());
    eprintln!("oracles agree: {agreed}");
    eprintln!("ical-rs matches: {matched}");
    eprintln!("ical-rs dissents: {}", dissent.len());

    for (start, rule, expected, ours) in &dissent {
        eprintln!("  {start}  {rule}\n    oracles: {expected}\n    ical-rs: {ours}");
    }
}

/// One answer file, keyed by `start<TAB>rule` and kept in file order.
struct Answers {
    order: Vec<(String, String)>,
    by_key: HashMap<String, String>,
}

fn read_answers(path: &str) -> Answers {
    let file = BufReader::new(File::open(path).unwrap_or_else(|e| panic!("open {path}: {e}")));

    let mut order = Vec::new();
    let mut by_key = HashMap::new();

    for line in file.lines() {
        let line = line.expect("read answer");
        let mut fields = line.splitn(3, '\t');
        let (Some(start), Some(rule)) = (fields.next(), fields.next()) else {
            continue;
        };
        let answer = fields.next().unwrap_or("").to_string();
        let key = format!("{start}\t{rule}");

        order.push((key.clone(), answer.clone()));
        by_key.insert(key, answer);
    }

    Answers { order, by_key }
}

/// Whether an answer is a marker (an error, a refusal, a timeout) rather than a
/// list of occurrences. An empty answer is an opinion: the rule yields nothing.
fn is_marker(answer: &str) -> bool {
    answer
        .chars()
        .next()
        .map(|first| first.is_ascii_uppercase())
        .unwrap_or(false)
}

/// A small linear congruential generator, so the random sweep is reproducible
/// with no dependency. The constants are Knuth's MMIX.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}
