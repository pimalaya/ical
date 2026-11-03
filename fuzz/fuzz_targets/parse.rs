#![no_main]

//! Coverage-guided fuzz target for the parser. Three oracles: parsing arbitrary
//! bytes must never panic, whatever parses must serialize to a byte-stable
//! fixpoint (its own output reparses identically), and the content decoders,
//! run under the detected CPU token, never panic and never grow a
//! quoted-printable value.

use ical::{
    component::IcalComponent,
    prop::IcalProp,
    tree::{
        cst::{IcalCst, IcalItem},
        value::cursor::IcalValueCursor,
    },
    value::IcalValue,
};
use libfuzzer_sys::fuzz_target;
use simd_cpu::SimdCpu;

fuzz_target!(
    init: {
        let _ = SimdCpu::install_detected();
    },
    |data: &[u8]| {
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
        for mut cal in IcalCst::parse_many(data).flatten() {
            let _ = cal.to_bytes();

            {
                let decoded = cal.decode();
                decode_binaries(&decoded.props, &decoded.components);
            }

            decode_lines(&mut cal);
        }
    }
);

/// Decodes every binary value of `props` and `components`, nested included.
fn decode_binaries(props: &[IcalProp<'_>], components: &[IcalComponent<'_>]) {
    for prop in props {
        if let IcalValue::Binary(binary) = &prop.value {
            let _ = binary.decode_base64();
        }
    }

    for component in components {
        decode_binaries(&component.props, &component.components);
    }
}

/// Decodes every line of `cst` through the quoted-printable and charset
/// decoders, nested components included.
fn decode_lines(cst: &mut IcalCst<'_>) {
    for item in &mut cst.items {
        match item {
            IcalItem::Prop(line) => {
                let cursor = IcalValueCursor { line };
                let decoded = cursor.quoted_printable();
                assert!(
                    decoded.len() <= cursor.bytes().len(),
                    "a decoded value grew"
                );
                let _ = cursor.charset();
            }
            IcalItem::Component(inner) => decode_lines(inner),
            IcalItem::Opaque(_) => {}
        }
    }
}
