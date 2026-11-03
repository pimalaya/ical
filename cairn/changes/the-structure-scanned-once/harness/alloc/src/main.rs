//! Allocations of one corpus pass and of one card: `ialloc`.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    fs,
    hint::black_box,
    path::Path,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
};

use ical::tree::cst::IcalCst;

struct Counting;

static ON: AtomicBool = AtomicBool::new(false);
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static REALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ON.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(layout.size(), Relaxed);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if ON.load(Relaxed) {
            REALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(size.saturating_sub(layout.size()), Relaxed);
        }
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn count(name: &str, f: impl FnOnce()) {
    ALLOCS.store(0, Relaxed);
    REALLOCS.store(0, Relaxed);
    BYTES.store(0, Relaxed);
    ON.store(true, Relaxed);
    f();
    ON.store(false, Relaxed);
    println!("{name:10} {:>9} allocs {:>7} reallocs {:>11} bytes", ALLOCS.load(Relaxed), REALLOCS.load(Relaxed), BYTES.load(Relaxed));
}

fn walk(p: &Path, out: &mut Vec<Vec<u8>>) {
    let mut es: Vec<_> = fs::read_dir(p).unwrap().flatten().map(|e| e.path()).collect();
    es.sort();
    for e in es {
        if e.is_dir() { walk(&e, out) } else if e.extension().is_some_and(|x| x == "ics") { out.push(fs::read(e).unwrap()) }
    }
}

fn main() {
    let mut corpus = Vec::new();
    walk(Path::new("/home/soywod/code/pimalaya/ical/tests/corpus"), &mut corpus);
    for (label, files) in [("corpus", corpus), ("card", vec![CAL.as_bytes().to_vec()])] {
        println!("== {label}");
        let mut csts = Vec::new();
        count("parse", || {
            for f in &files {
                csts.extend(IcalCst::parse_many(f.as_slice()).flatten());
            }
        });
        let mut cals = Vec::new();
        count("decode", || cals.extend(csts.iter().map(|c| c.decode())));
        count("to_bytes", || for c in &csts { black_box(c.to_bytes()); });
        count("str", || for c in &csts { black_box(c.to_string()); });
        count("to_string", || for c in &cals { black_box(c.to_string()); });
        count("encode", || for c in &cals { black_box(c.encode()); });
    }
}

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
