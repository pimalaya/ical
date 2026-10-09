---
cairn: delta
change: a-written-uri-reads-back
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A calendar address is a URI
In decoded-model. A `\` in a written URI or calendar user address SHALL go out as `%5C`, so a written value reads back and writes back the same; `,` and `;` still go out as held.

### Requirement: A written value never breaks its line
In parsing. Beside a line break, a backslash written into a URI or a calendar user address SHALL go out percent-encoded, as `%5C`.

## REMOVED Requirements
