---
cairn: change
id: repo-hygiene
status: landed
created: 2026-08-08
---

# Repo hygiene, to the Pimalaya guidelines

## Why

ical-rs is missing what every other Pimalaya repository carries: CONTRIBUTING.md (deviations only, the org-wide one lives in `.github`), SECURITY.md, deny.toml, and the settled design record. The CI workflow is missing the `audit` job vcard-rs runs. Cheap, and it unblocks reading the repository the way the guidelines expect.

## What

Bring the file set up to vcard-rs's, adapted to Cairn: the design record is cairn/spec/, not a docs/design.md, so CONTRIBUTING.md points there. Add the `audit` CI job. Done when the file set matches vcard-rs and `cargo deny check` passes.
