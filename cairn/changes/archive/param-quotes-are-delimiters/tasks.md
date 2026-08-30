---
cairn: tasks
change: param-quotes-are-delimiters
---

- [x] Strip a balanced surrounding pair in `unescape_param`, before the carets
- [x] Wrap the encoded text in a pair in `escape_param` where a delimiter needs it
- [x] Add `Escaper::has_param_quoting`, false for vCalendar 1.0
- [x] Delete the JSCalendar export's `unquoted` workaround
- [x] Pin the read, the write-back and the unbalanced quote, and that requoting is not a merge change
- [x] Fold the spec and log the change
