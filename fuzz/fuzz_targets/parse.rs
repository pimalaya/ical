#![no_main]

//! Coverage-guided fuzz target for the parser. Two oracles: parsing arbitrary
//! bytes must never panic, and whatever parses must serialize to a byte-stable
//! fixpoint (its own output reparses identically).

use ical::tree::cst::IcalCst;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(cst) = IcalCst::parse(data) {
        let bytes = cst.to_bytes();
        let _ = cst.decode();
        let _ = cst.to_string();

        let reparsed = IcalCst::parse(&bytes).expect("serialized output must reparse");
        assert_eq!(
            reparsed.to_bytes(),
            bytes,
            "serialization is not idempotent"
        );
    }

    // The multi-calendar path must not panic either.
    for cal in IcalCst::parse_many(data).flatten() {
        let _ = cal.to_bytes();
        let _ = cal.decode();
    }
});
