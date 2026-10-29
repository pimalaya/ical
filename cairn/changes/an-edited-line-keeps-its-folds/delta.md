---
cairn: delta
change: an-edited-line-keeps-its-folds
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: An encoded line is folded

A content line with no recorded shape, one encoded from the model or built by hand, SHALL be folded when longer than 75 octets (RFC 5545 3.1): a line break and a single space before every continuation, each physical line 75 octets at most, its line ending excluded and the continuation space included. A fold SHALL NOT land inside a UTF-8 sequence. The break SHALL be the line's own ending, `\r\n` unless the line ends on a bare `\n`.

A line an edit changed SHALL keep every recorded fold before the first byte the edit changed, and SHALL be folded from the last kept fold onward with its own folding whitespace and line ending, under the same 75-octet and UTF-8 rules.

A parsed line SHALL keep its recorded layout byte for byte, folded or not, however long: the fold is how the crate writes, never a rewrite of what it read.

A `QUOTED-PRINTABLE` line SHALL NOT be folded, since a fold after one of its `=` would read back as a soft break, and the encoding carries soft breaks of its own for a long value.

#### Scenario: A long description edited at its end
- GIVEN a parsed `DESCRIPTION` of 300 octets folded every 74 octets with a tab
- WHEN its last word is replaced by a longer one
- THEN every physical line before the one holding the edit comes out byte for byte, the rest is folded with a tab, no physical line exceeds 75 octets, and the output reparses to the edited value

## REMOVED Requirements
