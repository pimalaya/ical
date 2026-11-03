//! Codec harness: `icodec <work> <passes> [token]`, work one of `b64` (decode
//! an inline folded ATTACH of 36 KiB), `qp` (decode a soft-broken 1.0
//! QUOTED-PRINTABLE description), `cs` (the same through `charset`), `b64s`
//! and `qps` (one short value each, 300 times a pass). `token` installs the
//! detected CPU token first (new tree only).

use std::hint::black_box;

use ical::{
    component::vevent::VEVENT,
    prop::description::DESCRIPTION,
    tree::cst::IcalCst,
    value::{IcalValue, binary::IcalBinary},
};

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
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// A logical line folded at 75 octets as RFC 5545 3.1 does.
fn fold(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = String::new();
    let mut at = 0;
    let mut room = 75;
    while bytes.len() - at > room {
        out.push_str(&line[at..at + room]);
        out.push_str("\r\n ");
        at += room;
        room = 74;
    }
    out.push_str(&line[at..]);
    out.push_str("\r\n");
    out
}

/// Quoted-printable text with soft breaks every 76 octets at most.
fn quoted(text: &str) -> String {
    let mut encoded = String::new();
    for byte in text.bytes() {
        match byte {
            b'=' => encoded.push_str("=3D"),
            b'\n' => encoded.push_str("=0D=0A"),
            0x21..=0x7e | b' ' => encoded.push(byte as char),
            _ => encoded.push_str(&format!("={byte:02X}")),
        }
    }
    let mut out = String::new();
    let mut col = 0;
    let mut chars = encoded.as_str();
    while !chars.is_empty() {
        let take = if chars.starts_with('=') { 3 } else { 1 };
        if col + take > 75 {
            out.push_str("=\r\n");
            col = 0;
        }
        out.push_str(&chars[..take]);
        col += take;
        chars = &chars[take..];
    }
    out
}

fn attach_calendar(size: usize) -> String {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let bytes: Vec<u8> = (0..size).map(|_| rng.next() as u8).collect();
    let line = format!("ATTACH;FMTTYPE=image/png;ENCODING=BASE64;VALUE=BINARY:{}", base64(&bytes));
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//EN\r\nBEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n{}END:VEVENT\r\nEND:VCALENDAR\r\n",
        fold(&line)
    )
}

fn qp_calendar(paragraphs: usize) -> String {
    let text = "Réunion trimestrielle: l'équipe présente le bilan, les écarts et les décisions à prendre. Café et pâtisseries offerts.\n".repeat(paragraphs);
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:1.0\r\nBEGIN:VEVENT\r\nUID:1\r\nDTSTART:20260101T090000Z\r\nDESCRIPTION;ENCODING=QUOTED-PRINTABLE;CHARSET=UTF-8:{}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        quoted(&text)
    )
}

fn main() {
    let mut args = std::env::args().skip(1);
    let work = args.next().unwrap();
    let passes: usize = args.next().unwrap().parse().unwrap();

    if args.next().as_deref() == Some("token") {
        #[cfg(not(feature = "old"))]
        let _ = simd_cpu::SimdCpu::install_detected();
    }

    let (raw, reps) = match work.as_str() {
        "b64" => (attach_calendar(36 * 1024), 1),
        "b64s" => (attach_calendar(48), 300),
        "qp" | "cs" => (qp_calendar(32), 1),
        "qps" | "css" => (qp_calendar(1), 300),
        _ => panic!("unknown work {work}"),
    };

    let mut cst = IcalCst::parse(&raw).unwrap();

    if work.starts_with("b64") {
        let cal = cst.decode();
        let IcalValue::Binary(binary @ IcalBinary::Base64(_)) = &cal.components[0].props[2].value else {
            panic!("no inline binary");
        };
        for _ in 0..passes * reps {
            #[cfg(feature = "old")]
            black_box(black_box(binary).decode_base64().unwrap().unwrap());
            #[cfg(not(feature = "old"))]
            black_box(black_box(binary).decode_base64().unwrap());
        }
        return;
    }

    let event = cst.component_mut::<VEVENT>().unwrap();
    let cursor = event.prop_mut::<DESCRIPTION>().unwrap();
    for _ in 0..passes * reps {
        if work.starts_with("cs") {
            black_box(black_box(&cursor).charset());
        } else {
            black_box(black_box(&cursor).quoted_printable());
        }
    }
}
