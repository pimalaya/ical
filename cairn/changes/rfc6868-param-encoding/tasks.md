---
cairn: tasks
change: rfc6868-param-encoding
---

- [ ] Add an RFC 6868 decoder: `^n` to newline, `^^` to caret, `^'` to double quote, a caret before anything else kept verbatim
- [ ] Add the inverse encoder, and point the parameter codec at both instead of the text unescaper
- [ ] Stop applying the RFC 5545 3.3.11 text escapes to parameter values
- [ ] Prove a parameter round trips byte for byte, and that a lone caret is untouched
- [ ] Re-run the golden corpus and account for every fixture that moves
- [ ] Mirror the whole change in vcard-rs (RFC 6350 3.3), keeping the twins aligned
