// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PL-59.1: AXON sovereign script interpreter (bare-metal subset)
// Supports: print "text", let x = N, // comments, awp <payload>
#![allow(dead_code)]

pub const MAX_LINES: usize = 32;
pub const MAX_VARS: usize = 8;
pub const VAR_NAME_LEN: usize = 16;
pub const OUTPUT_BUF: usize = 512;

/// One interpreter variable slot
#[derive(Clone, Copy)]
pub struct AxVar {
    pub name: [u8; VAR_NAME_LEN],
    pub name_len: usize,
    pub value: i64,
}

impl AxVar {
    const fn empty() -> Self {
        AxVar { name: [0u8; VAR_NAME_LEN], name_len: 0, value: 0 }
    }
}

/// Interpreter result
pub struct AxResult {
    pub output: [u8; OUTPUT_BUF],
    pub output_len: usize,
    pub lines_executed: usize,
    pub error: bool,
    pub error_line: usize,
}

impl AxResult {
    fn new() -> Self {
        AxResult {
            output: [0u8; OUTPUT_BUF],
            output_len: 0,
            lines_executed: 0,
            error: false,
            error_line: 0,
        }
    }

    fn push_str(&mut self, s: &[u8]) {
        let mut i = 0;
        while i < s.len() && self.output_len < OUTPUT_BUF - 1 {
            self.output[self.output_len] = s[i];
            self.output_len += 1;
            i += 1;
        }
    }

    fn push_newline(&mut self) {
        if self.output_len < OUTPUT_BUF - 1 {
            self.output[self.output_len] = b'\n';
            self.output_len += 1;
        }
    }

    fn push_i64(&mut self, mut n: i64) {
        if n < 0 {
            self.push_str(b"-");
            n = n.wrapping_neg();
        }
        let mut tmp = [0u8; 20];
        let mut len = 0usize;
        if n == 0 { self.push_str(b"0"); return; }
        while n > 0 {
            tmp[len] = b'0' + (n % 10) as u8;
            len += 1;
            n /= 10;
        }
        // reverse
        let mut i = len;
        while i > 0 {
            i -= 1;
            if self.output_len < OUTPUT_BUF - 1 {
                self.output[self.output_len] = tmp[i];
                self.output_len += 1;
            }
        }
    }

    pub fn as_str(&self) -> &[u8] {
        &self.output[..self.output_len]
    }
}

// ── Parser helpers ─────────────────────────────────────────────────────────────

fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < s.len() && (s[start] == b' ' || s[start] == b'\t') { start += 1; }
    let mut end = s.len();
    while end > start && (s[end-1] == b' ' || s[end-1] == b'\t' || s[end-1] == b'\r' || s[end-1] == b'\n') { end -= 1; }
    &s[start..end]
}

fn starts_with(s: &[u8], prefix: &[u8]) -> bool {
    s.len() >= prefix.len() && &s[..prefix.len()] == prefix
}

fn parse_i64(s: &[u8]) -> Option<i64> {
    let s = trim(s);
    if s.is_empty() { return None; }
    let (neg, digits) = if s[0] == b'-' { (true, &s[1..]) } else { (false, s) };
    let mut val: i64 = 0;
    let mut i = 0;
    while i < digits.len() {
        if digits[i] < b'0' || digits[i] > b'9' { return None; }
        val = val * 10 + (digits[i] - b'0') as i64;
        i += 1;
    }
    Some(if neg { -val } else { val })
}

fn find_var(vars: &[AxVar], name: &[u8]) -> Option<i64> {
    let nlen = name.len();
    let mut i = 0;
    while i < vars.len() {
        if vars[i].name_len == nlen && &vars[i].name[..nlen] == name {
            return Some(vars[i].value);
        }
        i += 1;
    }
    None
}

fn set_var(vars: &mut [AxVar; MAX_VARS], name: &[u8], value: i64) {
    let nlen = name.len().min(VAR_NAME_LEN);
    // find existing
    let mut i = 0;
    while i < MAX_VARS {
        if vars[i].name_len == nlen && vars[i].name[..nlen] == name[..nlen] {
            vars[i].value = value;
            return;
        }
        i += 1;
    }
    // find empty slot
    let mut j = 0;
    while j < MAX_VARS {
        if vars[j].name_len == 0 {
            vars[j].name_len = nlen;
            let mut k = 0; while k < nlen { vars[j].name[k] = name[k]; k += 1; }
            vars[j].value = value;
            return;
        }
        j += 1;
    }
}

// ── Main interpreter ───────────────────────────────────────────────────────────

/// Execute AXON script bytes. Returns AxResult with output.
/// `awp_send`: optional callback for `awp` statements — (node_id, payload) -> bool
pub fn exec(
    script: &[u8],
    awp_node_id: u64,
    awp_send: Option<fn(u64, &[u8]) -> bool>,
) -> AxResult {
    let mut result = AxResult::new();
    let mut vars = [AxVar::empty(); MAX_VARS];

    // Split into lines
    let mut line_start = 0usize;
    let mut line_num = 0usize;

    while line_start <= script.len() && line_num < MAX_LINES {
        // Find end of line
        let mut line_end = line_start;
        while line_end < script.len() && script[line_end] != b'\n' { line_end += 1; }

        let raw_line = &script[line_start..line_end];
        let line = trim(raw_line);

        if !line.is_empty() {
            exec_line(line, &mut vars, &mut result, awp_node_id, awp_send, line_num);
            if result.error { return result; }
            result.lines_executed += 1;
        }

        line_start = line_end + 1;
        line_num += 1;
    }

    result
}

fn exec_line(
    line: &[u8],
    vars: &mut [AxVar; MAX_VARS],
    result: &mut AxResult,
    awp_node_id: u64,
    awp_send: Option<fn(u64, &[u8]) -> bool>,
    line_num: usize,
) {
    // Comment
    if starts_with(line, b"//") { return; }

    // print "text" or print varname
    if starts_with(line, b"print ") {
        let arg = trim(&line[6..]);
        if arg.len() >= 2 && arg[0] == b'"' && arg[arg.len()-1] == b'"' {
            // String literal
            result.push_str(&arg[1..arg.len()-1]);
            result.push_newline();
        } else {
            // Variable
            if let Some(val) = find_var(vars, arg) {
                result.push_i64(val);
                result.push_newline();
            } else {
                result.push_str(b"undefined: ");
                result.push_str(arg);
                result.push_newline();
            }
        }
        return;
    }

    // let x = N  or  let x = varname
    if starts_with(line, b"let ") {
        let rest = trim(&line[4..]);
        // find '='
        let mut eq = 0usize;
        while eq < rest.len() && rest[eq] != b'=' { eq += 1; }
        if eq >= rest.len() {
            result.error = true; result.error_line = line_num; return;
        }
        let name = trim(&rest[..eq]);
        let val_src = trim(&rest[eq+1..]);
        let value = if let Some(n) = parse_i64(val_src) {
            n
        } else if let Some(v) = find_var(vars, val_src) {
            v
        } else {
            result.error = true; result.error_line = line_num; return;
        };
        set_var(vars, name, value);
        return;
    }

    // awp <payload>
    if starts_with(line, b"awp ") {
        let payload = trim(&line[4..]);
        if let Some(send) = awp_send {
            let sent = send(awp_node_id, payload);
            if sent {
                result.push_str(b"awp: sent ");
                result.push_str(payload);
                result.push_newline();
            } else {
                result.push_str(b"awp: not live\n");
            }
        } else {
            result.push_str(b"awp: no transport\n");
        }
        return;
    }

    // fn declarations — skip (no execution context needed for flat scripts)
    if starts_with(line, b"fn ") || starts_with(line, b"return ") { return; }

    // Unknown — report
    result.push_str(b"unknown: ");
    result.push_str(line);
    result.push_newline();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_string_literal() {
        let script = b"print \"hello sovereign\"\n";
        let r = exec(script, 0, None);
        assert!(!r.error);
        assert_eq!(&r.output[..r.output_len], b"hello sovereign\n");
    }

    #[test]
    fn let_and_print_var() {
        let script = b"let x = 42\nprint x\n";
        let r = exec(script, 0, None);
        assert!(!r.error);
        assert_eq!(&r.output[..r.output_len], b"42\n");
    }

    #[test]
    fn comments_ignored() {
        let script = b"// this is a comment\nprint \"ok\"\n";
        let r = exec(script, 0, None);
        assert!(!r.error);
        assert_eq!(&r.output[..r.output_len], b"ok\n");
    }

    #[test]
    fn let_from_var() {
        let script = b"let a = 10\nlet b = a\nprint b\n";
        let r = exec(script, 0, None);
        assert!(!r.error);
        assert_eq!(&r.output[..r.output_len], b"10\n");
    }

    #[test]
    fn multi_line_script() {
        let script = b"let n = 7\nprint \"aiXos\"\nprint n\n";
        let r = exec(script, 0, None);
        assert!(!r.error);
        assert_eq!(r.lines_executed, 3);
    }

    #[test]
    fn awp_no_transport() {
        let script = b"awp hello\n";
        let r = exec(script, 1, None);
        assert!(!r.error);
        assert!(r.output_len > 0);
    }
}
