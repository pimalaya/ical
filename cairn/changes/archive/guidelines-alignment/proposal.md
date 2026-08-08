---
cairn: change
id: guidelines-alignment
status: landed
created: 2026-08-08
---

# Align the whole repository with the Pimalaya guidelines

## Why

The org-wide guidelines (.github/GUIDELINES.md) are a living draft with stable rule ids, and they say plainly that a repository contradicting them needs realigning. ical-rs contradicted them in several places at once: a README carrying API snippets, a manifest whose fields and dependencies were out of order, bare comments, a feature gating no dependency, and a public type with no domain prefix.

## What

A conformance pass over every scope that applies to a library, fixing what fails rather than filing it.

