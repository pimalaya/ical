//! Differential run: ical-rs (working tree) against ical-old (the baseline
//! copy), over tests/corpus and random edits of it.
//!
//! Each version produces a list of observations for one input (trees and
//! errors as `Debug`, bytes, text, decoded models, encodes, cursor reads,
//! then the same after an edit of component 2 on every line); the two lists
//! must be equal.

use std::{fs, path::Path};

fn walk(p: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let mut es: Vec<_> = fs::read_dir(p).unwrap().flatten().map(|e| e.path()).collect();
    es.sort();
    for e in es {
        if e.is_dir() {
            walk(&e, out)
        } else if e.extension().is_some_and(|x| x == "ics") {
            out.push((e.display().to_string(), fs::read(&e).unwrap()))
        }
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const ALPHABET: &[&[u8]] = &[
    b"\r",
    b"\n",
    b" ",
    b"\t",
    b":",
    b";",
    b",",
    b"\"",
    b"=",
    b"\\",
    b"^",
    b"ENCODING=QUOTED-PRINTABLE",
    b"QUOTED-PRINTABLE",
    b"CHARSET=",
    b"BEGIN:VCALENDAR",
    b"END:VCALENDAR",
    b"BEGIN:VEVENT",
    b"END:VEVENT",
    b"BEGIN:VALARM",
    b"END:VALARM",
    b"BEGIN:X-THING",
    b"END:X-THING",
    b"VERSION:1.0",
    b"VERSION:2.0",
    b"\xc3\xa9",
    b"\xff",
    b"\xe2\x82",
    b"\r\n ",
    b"\r\n\t",
    b"\n ",
    b"=\r\n",
    b"==\r\n",
    b"\\n",
    b"\\N",
    b"\\,",
    b"\\;",
    b"\\\\",
    b"^n",
    b"^'",
    b"^^",
    b";VALUE=URI",
    b";VALUE=DATE",
    b";VALUE=TEXT",
    b";TZID=Europe/Paris",
    b";X-P=\"a,b\"",
    b";CN=\"A:B;C\"",
    b"\r\nGEO:1.5;2.5\r\n",
    b"\r\nREQUEST-STATUS:2.0;Success\\, ok;x,y\r\n",
    b"\r\nRRULE:FREQ=WEEKLY;BYDAY=MO,WE\r\n",
    b"\r\nX-FOO;X-BAR=1:a;b,c\\;d\r\n",
    b"\r\nATTENDEE;CN=\"A:B\";ROLE=CHAIR:mailto:a@b\r\n",
    b"\r\nCATEGORIES:a,b\\,c\r\n",
    b"\r\nRDATE;VALUE=PERIOD:19970101T180000Z/PT5H30M,19970102T180000Z/PT1H\r\n",
    b"\r\nATTACH;ENCODING=BASE64;VALUE=BINARY:AAAA\r\n",
    b"\r\nDESCRIPTION;ENCODING=QUOTED-PRINTABLE:a=\r\n=41=\r\nb\r\n",
    b"\r\nSUMMARY;LANGUAGE=en:x\r\n y\r\n\tz\r\n",
];

fn mutate(rng: &mut Rng, file: &[u8]) -> Vec<u8> {
    let mut bytes = if file.len() > 4096 {
        let at = rng.below(file.len() - 4096);
        file[at..at + 4096].to_vec()
    } else {
        file.to_vec()
    };

    for _ in 0..1 + rng.below(4) {
        let at = if bytes.is_empty() {
            0
        } else {
            rng.below(bytes.len())
        };
        match rng.below(5) {
            0 if !bytes.is_empty() => {
                bytes.remove(at);
            }
            1 if !bytes.is_empty() => {
                let piece = ALPHABET[rng.below(ALPHABET.len())];
                bytes.splice(at..at + 1, piece.iter().copied());
            }
            2 | 3 => {
                let piece = ALPHABET[rng.below(ALPHABET.len())];
                bytes.splice(at..at, piece.iter().copied());
            }
            _ if bytes.len() > 2 => {
                let from = rng.below(bytes.len() - 1);
                let len = 1 + rng.below((bytes.len() - from).min(200));
                let chunk = bytes[from..from + len].to_vec();
                bytes.splice(at..at, chunk);
            }
            _ => {}
        }
    }

    bytes
}

macro_rules! observe {
    ($name:ident, $krate:ident) => {
        mod $name {
            use std::borrow::Cow;

            use $krate::{
                component::{IcalComponent, IcalComponentName},
                ical::Ical,
                param::IcalParam,
                prop::{IcalProp, IcalPropName},
                tree::{
                    codec::mode::Escaper,
                    cst::{IcalCst, IcalItem},
                    line::IcalLine,
                    value::cursor::IcalValueCursor,
                },
                value::{
                    IcalValue,
                    binary::IcalBinary,
                    boolean::IcalBoolean,
                    cal_address::IcalCalAddress,
                    datetime::{IcalDate, IcalDateTime, IcalDateTimeList, IcalTime},
                    duration::IcalDuration,
                    float::IcalFloat,
                    geo::IcalGeo,
                    integer::IcalInteger,
                    period::IcalPeriod,
                    recur::IcalRecur,
                    request_status::IcalRequestStatus,
                    text::{IcalText, IcalTextList},
                    uri::IcalUri,
                    utc_offset::IcalUtcOffset,
                },
            };

            const SALTS: &[&str] = &[
                ",", ";", "\\", "\n", "\r\n", "^", "\"", ":", "\u{e9}", "\\n", "=", " ", "\t",
                "a,b;c\\d\ne", "^'\"^^", "=\r\n", "\u{1F600}x", "\r",
            ];

            /// Every string of a model salted, the salt and where it goes
            /// (front, back, both) picked by a running count, so both versions
            /// salt alike.
            struct Salt(usize);

            impl Salt {
                fn s(&mut self, text: &mut Cow<'_, str>) {
                    self.0 += 1;
                    let salt = SALTS[self.0 % SALTS.len()];
                    let mut owned = text.to_string();
                    match self.0 / SALTS.len() % 3 {
                        0 => owned.insert_str(0, salt),
                        1 => owned.push_str(salt),
                        _ => {
                            owned.insert_str(0, salt);
                            owned.push_str(salt);
                        }
                    }
                    *text = Cow::Owned(owned);
                }

                fn list(&mut self, list: &mut Vec<Cow<'_, str>>) {
                    if list.is_empty() {
                        list.push(Cow::Borrowed(""));
                    }
                    for text in list {
                        self.s(text);
                    }
                }

                fn param(&mut self, param: &mut IcalParam<'_>) {
                    match param {
                        IcalParam::AltRep(v)
                        | IcalParam::Cn(v)
                        | IcalParam::CuType(v)
                        | IcalParam::Dir(v)
                        | IcalParam::Encoding(v)
                        | IcalParam::FmtType(v)
                        | IcalParam::FbType(v)
                        | IcalParam::Language(v)
                        | IcalParam::PartStat(v)
                        | IcalParam::Range(v)
                        | IcalParam::Related(v)
                        | IcalParam::RelType(v)
                        | IcalParam::Role(v)
                        | IcalParam::Rsvp(v)
                        | IcalParam::SentBy(v)
                        | IcalParam::TzId(v)
                        | IcalParam::Value(v)
                        | IcalParam::Display(v)
                        | IcalParam::Email(v)
                        | IcalParam::Label(v)
                        | IcalParam::Order(v)
                        | IcalParam::Schema(v)
                        | IcalParam::Derived(v)
                        | IcalParam::ScheduleAgent(v)
                        | IcalParam::ScheduleForceSend(v)
                        | IcalParam::ScheduleStatus(v)
                        | IcalParam::LinkRel(v)
                        | IcalParam::Gap(v)
                        | IcalParam::Charset(v) => self.s(v),
                        IcalParam::DelegatedFrom(vs)
                        | IcalParam::DelegatedTo(vs)
                        | IcalParam::Member(vs)
                        | IcalParam::Feature(vs) => self.list(vs),
                        IcalParam::Unknown { name, values } => {
                            if self.0 % 5 == 0 {
                                self.s(name);
                            }
                            self.list(values);
                        }
                    }
                }

                fn value(&mut self, value: &mut IcalValue<'_>) {
                    match value {
                        IcalValue::Binary(IcalBinary::Uri(v) | IcalBinary::Base64(v))
                        | IcalValue::Boolean(IcalBoolean(v))
                        | IcalValue::CalAddress(IcalCalAddress(v))
                        | IcalValue::Date(IcalDate(v))
                        | IcalValue::DateTime(IcalDateTime(v))
                        | IcalValue::Duration(IcalDuration(v))
                        | IcalValue::Float(IcalFloat(v))
                        | IcalValue::Integer(IcalInteger(v))
                        | IcalValue::Period(IcalPeriod(v))
                        | IcalValue::Recur(IcalRecur(v))
                        | IcalValue::Text(IcalText(v))
                        | IcalValue::Time(IcalTime(v))
                        | IcalValue::Uri(IcalUri(v))
                        | IcalValue::UtcOffset(IcalUtcOffset(v)) => self.s(v),
                        IcalValue::DateTimeList(IcalDateTimeList(vs))
                        | IcalValue::TextList(IcalTextList(vs)) => self.list(vs),
                        IcalValue::Geo(IcalGeo {
                            latitude,
                            longitude,
                        }) => {
                            self.s(latitude);
                            self.s(longitude);
                        }
                        IcalValue::RequestStatus(IcalRequestStatus {
                            code,
                            description,
                            extra,
                        }) => {
                            self.s(code);
                            self.s(description);
                            self.s(extra);
                        }
                        IcalValue::Unknown(unknown) => {
                            if unknown.components.is_empty() {
                                unknown.components.push(Vec::new());
                            }
                            for component in &mut unknown.components {
                                self.list(component);
                            }
                        }
                    }
                }

                fn prop(&mut self, prop: &mut IcalProp<'_>) {
                    if let IcalPropName::Unknown(name) = &mut prop.name {
                        if self.0 % 7 == 0 {
                            self.s(name);
                        }
                    }
                    for param in &mut prop.params {
                        self.param(param);
                    }
                    self.value(&mut prop.value);
                }

                fn component(&mut self, component: &mut IcalComponent<'_>) {
                    if let IcalComponentName::Unknown(name) = &mut component.name {
                        if self.0 % 7 == 0 {
                            self.s(name);
                        }
                    }
                    for prop in &mut component.props {
                        self.prop(prop);
                    }
                    for child in &mut component.components {
                        self.component(child);
                    }
                }
            }

            fn salted(cal: &Ical<'_>, obs: &mut Vec<String>) {
                let mut cal = cal.clone();
                let mut salt = Salt(0);
                for prop in &mut cal.props {
                    salt.prop(prop);
                }
                for component in &mut cal.components {
                    salt.component(component);
                }
                model(&cal, obs);

                cal.version = match cal.version {
                    $krate::version::IcalVersion::V1_0 => $krate::version::IcalVersion::V2_0,
                    $krate::version::IcalVersion::V2_0 => $krate::version::IcalVersion::V1_0,
                };
                model(&cal, obs);
            }

            fn lines<'c, 'a>(cst: &'c mut IcalCst<'a>, out: &mut Vec<&'c mut IcalLine<'a>>) {
                if let Some(begin) = cst.begin.as_mut() {
                    out.push(begin);
                }
                for item in cst.items.iter_mut() {
                    match item {
                        IcalItem::Prop(line) => out.push(line),
                        IcalItem::Component(child) => lines(child, out),
                        IcalItem::Opaque(_) => {}
                    }
                }
                if let Some(end) = cst.end.as_mut() {
                    out.push(end);
                }
            }

            fn reads(line: &mut IcalLine<'_>, obs: &mut Vec<String>) {
                obs.push(format!("raw {:?} {:?}", line.raw_value(), line.raw_value_str()));
                obs.push(format!(
                    "count {} {:?} {:?} {:?}",
                    line.value.component_count(),
                    line.value.decode_component(0),
                    line.value.decode_component(1),
                    line.value.decode_component(2),
                ));
                obs.push(format!("bytes {:?}", line.value.decode_bytes()));
                let cursor = IcalValueCursor { line };
                obs.push(format!("text {:?}", cursor.text()));
                obs.push(format!("cbytes {:?}", cursor.bytes()));
                obs.push(format!("list {:?}", cursor.list()));
                for i in [0, 1, 2, 7] {
                    obs.push(format!("component {i} {:?}", cursor.component(i)));
                }
                obs.push(format!("qp {:?}", cursor.quoted_printable()));
                obs.push(format!("charset {:?}", cursor.charset()));
            }

            fn model(cal: &Ical<'_>, obs: &mut Vec<String>) {
                obs.push(format!("model {cal:?}"));
                obs.push(format!("model text {}", cal));
                let encoded = cal.encode();
                obs.push(format!("encoded {encoded:?}"));
                obs.push(format!("encoded bytes {:?}", encoded.to_bytes()));
                for prop in &cal.props {
                    for escaper in [Escaper::Modern, Escaper::V1_0] {
                        let line = prop.encode(escaper);
                        obs.push(format!("prop {line:?} {line}"));
                    }
                }
                for component in &cal.components {
                    for escaper in [Escaper::Modern, Escaper::V1_0] {
                        let cst = component.encode(escaper);
                        obs.push(format!("component {cst:?} {:?}", cst.to_bytes()));
                    }
                }
            }

            fn calendar(cst: &mut IcalCst<'_>, obs: &mut Vec<String>) {
                obs.push(format!("tree {cst:?}"));
                obs.push(format!("tree bytes {:?}", cst.to_bytes()));
                obs.push(format!("tree text {}", cst));
                obs.push(format!("version {:?}", cst.version()));
                model(&cst.decode(), obs);
                salted(&cst.decode(), obs);

                let mut all = Vec::new();
                lines(cst, &mut all);
                for line in all {
                    reads(line, obs);
                }

                let mut all = Vec::new();
                lines(cst, &mut all);
                for line in all {
                    line.value.set_component(2, &["x,y", "z;"]);
                }

                obs.push(format!("edited tree {cst:?}"));
                obs.push(format!("edited bytes {:?}", cst.to_bytes()));
                obs.push(format!("edited text {}", cst));
                model(&cst.decode(), obs);

                let mut all = Vec::new();
                lines(cst, &mut all);
                for line in all {
                    reads(line, obs);
                }
            }

            pub fn observe(input: &[u8]) -> Vec<String> {
                let mut obs = Vec::new();

                match IcalCst::parse(input) {
                    Ok(mut cst) => calendar(&mut cst, &mut obs),
                    Err(e) => obs.push(format!("parse error {e:?}")),
                }

                for item in IcalCst::parse_many(input) {
                    match item {
                        Ok(mut cst) => calendar(&mut cst, &mut obs),
                        Err(e) => obs.push(format!("many error {e:?}")),
                    }
                }

                let mut recovery = IcalCst::parse_recovering(input);
                obs.push(format!("recovery {:?}", recovery.problems));
                obs.push(format!("recovery bytes {:?}", recovery.to_bytes()));
                for cst in &mut recovery.calendars {
                    calendar(cst, &mut obs);
                }

                obs
            }
        }
    };
}

observe!(new, ical);
observe!(old, ical_old);

fn check(name: &str, input: &[u8]) {
    let a = old::observe(input);
    let b = new::observe(input);

    if a != b {
        let dir = "/tmp/claude-1000/-home-soywod-code-pimalaya/763c3b3e-4fde-4255-978b-60f2447b812a/scratchpad/diff/failures";
        fs::create_dir_all(dir).unwrap();
        let path = format!("{dir}/fail-{}.ics", a.len() * 31 + input.len());
        fs::write(&path, input).unwrap();

        let at = a.iter().zip(&b).position(|(x, y)| x != y).unwrap_or(a.len().min(b.len()));
        let (x, y) = (
            a.get(at).map_or("<none>", |s| s.as_str()),
            b.get(at).map_or("<none>", |s| s.as_str()),
        );
        let first = x.bytes().zip(y.bytes()).position(|(p, q)| p != q).unwrap_or(x.len().min(y.len()));
        let from = first.saturating_sub(300);
        let cut = |s: &str| String::from_utf8_lossy(&s.as_bytes()[from.min(s.len())..(first + 300).min(s.len())]).into_owned();
        panic!(
            "{name}: differs at observation {at} (old {} new {}) byte {first}, saved {path}\nold: {}\nnew: {}",
            a.len(),
            b.len(),
            cut(x),
            cut(y),
        );
    }
}

fn main() {
    let mut a = std::env::args().skip(1);
    let rounds: usize = a.next().map_or(300, |n| n.parse().unwrap());
    let seed: u64 = a.next().map_or(0x9E37_79B9_7F4A_7C15, |n| n.parse().unwrap());

    let mut corpus = Vec::new();
    walk(Path::new("/home/soywod/code/pimalaya/ical/tests/corpus"), &mut corpus);

    for (name, file) in &corpus {
        check(name, file);
    }
    eprintln!("corpus: {} files agree", corpus.len());

    let mut rng = Rng(seed);
    let mut inputs = 0;

    for round in 0..rounds {
        for (name, file) in &corpus {
            let input = mutate(&mut rng, file);
            check(&format!("round {round} {name}"), &input);
            inputs += 1;
        }
        if round % 50 == 49 {
            eprintln!("round {}: {inputs} inputs agree", round + 1);
        }
    }

    eprintln!("done: {} corpus files, {inputs} edited inputs agree", corpus.len());
}
