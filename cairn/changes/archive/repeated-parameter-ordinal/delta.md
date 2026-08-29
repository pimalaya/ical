---
cairn: delta
change: repeated-parameter-ordinal
---

## ADDED Requirements

### Requirement: Repeated parameters

A property carrying one parameter name more than once SHALL have each occurrence matched with the occurrence at the same position on the other side, and a parameter action SHALL address the occurrence it named rather than the first of that name. Two actions on two different occurrences SHALL NOT collide.

#### Scenario: A line carrying one name twice

- GIVEN a property written with the same parameter name twice
- WHEN a calendar is merged with itself against itself
- THEN no action and no collision is reported
