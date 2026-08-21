---
cairn: tasks
change: module-paths
---

- [x] tree/param: lens and node public, the two doc-inlined globs gone
- [x] tree/prop: cardinality, lens and spec public, the two globs and the spec re-export gone
- [x] tree/value: cursor and node public, the two globs gone, the codec modules left private
- [x] tree/component: lens and spec public, the glob and the spec re-export gone
- [x] recur: the validate re-export gone, both types named through recur::validate
- [x] every use statement in src, tests, examples and benches on the owning module's path
- [x] every intra-doc link on the new paths, including the four module headers that linked bare names
- [x] cargo build (bare core and all features), cargo test --all-features, doctests, cargo fmt, cargo doc with no warnings
- [x] CHANGELOG: the path change under Unreleased / Changed
