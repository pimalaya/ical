#![cfg(all(feature = "parser", feature = "quoted-printable", feature = "base64"))]
//! The content decoders under the detected CPU token decode every corpus value
//! as the codecs' portable baseline does.
//!
//! The token is process-wide, so this test has its own binary: installing it
//! here leaves the other test binaries on the baseline.

use std::{
    fs,
    path::{Path, PathBuf},
};

use base64_codec::codec::Base64Codec;
use ical::{
    component::IcalComponent,
    prop::IcalProp,
    tree::{
        cst::{IcalCst, IcalItem},
        line::IcalLine,
        param::node::IcalParamNode,
        value::cursor::IcalValueCursor,
    },
    value::{IcalValue, binary::IcalBinary},
};
use quoted_printable_codec::codec::QuotedPrintableCodec;
use simd_cpu::SimdCpu;

#[test]
fn the_detected_token_decodes_as_the_baseline() {
    let _ = SimdCpu::install_detected();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let (mut quoted, mut binary) = (0, 0);

    for path in fixtures(&root) {
        let input = fs::read(&path).unwrap();

        for cal in IcalCst::parse_many(&input).flatten() {
            let mut cal = cal.into_static();

            for line in lines(&mut cal) {
                if !line.params.iter().any(declares_quoted_printable) {
                    continue;
                }

                let cursor = IcalValueCursor { line };
                let mut expected = Vec::new();
                QuotedPrintableCodec::BASELINE.decode_vec(&cursor.bytes(), &mut expected);
                assert_eq!(cursor.quoted_printable(), expected, "{path:?}");
                quoted += 1;
            }

            let decoded = cal.decode();

            for value in binaries(&decoded.props, &decoded.components) {
                let IcalBinary::Base64(data) = value else {
                    continue;
                };

                let mut expected = Vec::new();
                let clean = Base64Codec::BASELINE.decode_vec(data.as_bytes(), &mut expected);
                assert_eq!(value.decode_base64(), Some((expected, clean)), "{path:?}");
                binary += 1;
            }
        }
    }

    assert!(quoted > 0 && binary > 0, "{quoted} quoted, {binary} binary");
}

/// Every .ics file under `dir`, at any depth.
fn fixtures(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();

        if path.is_dir() {
            paths.extend(fixtures(&path));
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("ics") {
            paths.push(path);
        }
    }

    paths
}

/// Every property line of a calendar, its nested components included.
fn lines<'c, 'a>(cst: &'c mut IcalCst<'a>) -> Vec<&'c mut IcalLine<'a>> {
    cst.items
        .iter_mut()
        .flat_map(|item| match item {
            IcalItem::Prop(line) => vec![line],
            IcalItem::Component(inner) => lines(inner),
            IcalItem::Opaque(_) => Vec::new(),
        })
        .collect()
}

/// Every binary value of `props` and of `components`, nested ones included.
fn binaries<'c, 'a>(
    props: &'c [IcalProp<'a>],
    components: &'c [IcalComponent<'a>],
) -> Vec<&'c IcalBinary<'a>> {
    let own = props.iter().filter_map(|prop| match &prop.value {
        IcalValue::Binary(binary) => Some(binary),
        _ => None,
    });

    let nested = components
        .iter()
        .flat_map(|component| binaries(&component.props, &component.components));

    own.chain(nested).collect()
}

/// Whether a parameter is `ENCODING=QUOTED-PRINTABLE` or the bare 1.0 token.
fn declares_quoted_printable(param: &IcalParamNode<'_>) -> bool {
    let param = param.to_string();
    param.eq_ignore_ascii_case("QUOTED-PRINTABLE")
        || param.eq_ignore_ascii_case("ENCODING=QUOTED-PRINTABLE")
}
