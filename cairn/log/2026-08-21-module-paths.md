---
cairn: log
change: module-paths
landed: 2026-08-21
---

# Put the lens, spec, node and cursor types on their real module paths

Six submodules under tree/ were private, their contents re-exported into the parent with `#[doc(inline)] pub use ...::*`. That gave each type two names: the real one, which nobody could write because the module was private, and the flattened one everybody did write. naming-005 asks for the opposite, and vcard-rs settled the same question in its 0.2.0, so this crate was the odd one out.

The modules are public now and the re-exports are gone. `tree::param::lens`, `tree::param::node`, `tree::prop::cardinality`, `tree::prop::lens`, `tree::prop::spec`, `tree::value::cursor` and `tree::value::node` are part of the public path, as are `tree::component::lens` and `tree::component::spec`. recur.rs had a smaller version of the same, a `pub use self::validate::{IcalRecurPart, IcalRecurRuleProblem}` out of an already-public module; both types are now named through `recur::validate`.

Eleven public paths moved, no type was renamed, and nothing about the crate's behaviour changed. The 976 references across 156 files were rewritten mechanically and the compiler checked the result: bare-core and all-features builds, seventeen test binaries, the doctests, cargo fmt and a cargo doc with no warnings all pass. Four module headers linked their contract types by bare name, which stopped resolving the moment the re-export went, so they now link the owning module.

The fifteen value-codec modules under tree/value stay private on purpose. They hold `Codec` impls and no types, so publishing them would add paths with nothing at the end of them.

Capabilities moved: none. cairn/spec names these types without paths, so no delta was needed.
