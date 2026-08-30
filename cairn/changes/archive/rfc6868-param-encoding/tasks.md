---
cairn: tasks
change: rfc6868-param-encoding
---

- [x] Add an RFC 6868 decoder: `^n` to newline, `^^` to caret, `^'` to double quote, a caret before anything else kept verbatim
- [x] Add the inverse encoder, and point the parameter codec at both instead of the text unescaper
- [x] Stop applying the RFC 5545 3.3.11 text escapes to parameter values
- [x] Key both halves on the version, RFC 6868 updating RFC 5545 alone, which means a parameter node carries an escaper
- [x] Prove a parameter round trips byte for byte, and that a lone caret is untouched
- [x] Re-run the golden corpus and account for every fixture that moves (none did)
- [x] Compare parameters on their raw nodes in the merge, a decoded one reading its first value alone
- [x] Mirror the whole change in vcard-rs (RFC 6350 3.3), keeping the twins aligned
