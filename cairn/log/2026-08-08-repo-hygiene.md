---
cairn: log
change: repo-hygiene
landed: 2026-08-08
---

# Repo hygiene, to the Pimalaya guidelines

The file set now matches vcard-rs's, with Cairn standing in for `docs/`. Added `CONTRIBUTING.md` (deviations only, pointing at the `src/lib.rs` header and at `cairn/`, and documenting the fixture corpus, the recurrence corpus and the no-default-features build check), `SECURITY.md`, `deny.toml` (the same source and licence allowlists vcard-rs uses), and `.github/workflows/audit.yml` calling the shared `pimalaya/nix` audit workflow.

`cargo deny check` was red on first run: RUSTSEC-2026-0204, an invalid pointer dereference in `crossbeam-epoch` 0.9.18, reached through criterion's rayon. It is a dev-dependency and never ships, but the lockfile is committed, so it was bumped to 0.9.20 rather than ignored. All four `cargo deny` sections are green.

Capabilities moved: none. This change moved files, not behaviour.
