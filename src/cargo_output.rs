//! Extract executable paths from Cargo's JSON output without external crates.
//!
//! This is a deliberately small parser for the zero-dependency prototype. It
//! validates an entire object-shaped JSON line, but only interprets the top-level
//! `reason` and `executable` fields. Other values are parsed and discarded. It
//! does not infer paths from filenames, nested diagnostics, or target metadata.

use std::path::PathBuf;

const MAX_JSON_LINE_BYTES: usize = 16 * 1024 * 1024;
const MAX_DEPTH: usize = 64;

/// Return an executable reported by a `compiler-artifact` Cargo message.
///
/// Fresh artifacts count: Cargo's `fresh` field does not change the meaning of
/// `executable`. Non-object text is ignored. Object-shaped lines must be valid
/// JSON, with no trailing data; duplicate selected fields are rejected. Paths
/// are returned as reported, without normalization or filesystem access.
pub fn executable_from_message(line: &str) -> Result<Option<PathBuf>, String> {
    let trimmed = line.trim_start_matches([' ', '\t', '\r', '\n']);
    if !trimmed.starts_with('{') {
        return Ok(None);
    }
    if line.len() > MAX_JSON_LINE_BYTES {
        return Err("Cargo JSON message exceeds the 16 MiB inspection limit".into());
    }
    let mut parser = Parser { text: line, pos: 0 };
    parser.whitespace();
    parser.expect(b'{')?;
    parser.whitespace();
    let mut reason: Option<String> = None;
    let mut executable: Option<Option<String>> = None;
    if !parser.consume(b'}') {
        loop {
            let key = parser.string()?;
            parser.whitespace();
            parser.expect(b':')?;
            parser.whitespace();
            match key.as_str() {
                "reason" => {
                    if reason.is_some() {
                        return Err(parser.error("duplicate top-level reason field"));
                    }
                    reason = Some(parser.string()?);
                }
                "executable" => {
                    if executable.is_some() {
                        return Err(parser.error("duplicate top-level executable field"));
                    }
                    executable = Some(if parser.peek() == Some(b'n') {
                        parser.literal(b"null")?;
                        None
                    } else {
                        Some(parser.string()?)
                    });
                }
                _ => parser.value(1)?,
            }
            parser.whitespace();
            if parser.consume(b'}') {
                break;
            }
            parser.expect(b',')?;
            parser.whitespace();
        }
    }
    parser.whitespace();
    if parser.pos != line.len() {
        return Err(parser.error("trailing data after the JSON object"));
    }
    if reason.as_deref() != Some("compiler-artifact") {
        return Ok(None);
    }
    match executable.flatten() {
        Some(path) if path.is_empty() => Err("Cargo reported an empty executable path".into()),
        Some(path) => Ok(Some(PathBuf::from(path))),
        None => Ok(None),
    }
}

struct Parser<'a> {
    text: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn error(&self, message: &str) -> String {
        format!("invalid Cargo JSON at byte {}: {message}", self.pos)
    }

    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }

    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.pos += 1;
        }
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.consume(byte) {
            Ok(())
        } else {
            Err(self.error(&format!("expected {:?}", char::from(byte))))
        }
    }

    fn literal(&mut self, literal: &[u8]) -> Result<(), String> {
        if self.text.as_bytes()[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(())
        } else {
            Err(self.error("invalid JSON literal"))
        }
    }

    fn hex4(&mut self) -> Result<u16, String> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let digit = match self.peek() {
                Some(b'0'..=b'9') => self.text.as_bytes()[self.pos] - b'0',
                Some(b'a'..=b'f') => self.text.as_bytes()[self.pos] - b'a' + 10,
                Some(b'A'..=b'F') => self.text.as_bytes()[self.pos] - b'A' + 10,
                _ => return Err(self.error("expected four hexadecimal digits in Unicode escape")),
            };
            self.pos += 1;
            value = value * 16 + u16::from(digit);
        }
        Ok(value)
    }

    fn unicode_escape(&mut self) -> Result<char, String> {
        let first = self.hex4()?;
        let scalar = if (0xd800..=0xdbff).contains(&first) {
            self.expect(b'\\')?;
            self.expect(b'u')?;
            let second = self.hex4()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(self.error("a high surrogate must be followed by a low surrogate"));
            }
            0x10000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
        } else {
            u32::from(first)
        };
        char::from_u32(scalar).ok_or_else(|| self.error("unpaired Unicode surrogate"))
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut result = String::new();
        let mut start = self.pos;
        loop {
            match self.peek() {
                Some(b'"') => {
                    result.push_str(&self.text[start..self.pos]);
                    self.pos += 1;
                    return Ok(result);
                }
                Some(b'\\') => {
                    result.push_str(&self.text[start..self.pos]);
                    self.pos += 1;
                    let escaped = self.peek().ok_or_else(|| self.error("unfinished escape"))?;
                    self.pos += 1;
                    let decoded = match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{0008}',
                        b'f' => '\u{000c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => self.unicode_escape()?,
                        _ => return Err(self.error("unknown string escape")),
                    };
                    result.push(decoded);
                    start = self.pos;
                }
                Some(0..=31) => return Err(self.error("unescaped control character in string")),
                Some(_) => self.pos += 1,
                None => return Err(self.error("unterminated JSON string")),
            }
        }
    }

    fn number(&mut self) -> Result<(), String> {
        self.consume(b'-');
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(self.error("invalid JSON number")),
        }
        if self.consume(b'.') {
            self.digits()?;
        }
        if self.consume(b'e') || self.consume(b'E') {
            if !self.consume(b'+') {
                self.consume(b'-');
            }
            self.digits()?;
        }
        Ok(())
    }

    fn digits(&mut self) -> Result<(), String> {
        let start = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(self.error("expected at least one decimal digit"));
        }
        Ok(())
    }

    fn value(&mut self, depth: usize) -> Result<(), String> {
        match self.peek() {
            Some(b'{') => {
                self.container_depth(depth)?;
                self.pos += 1;
                self.whitespace();
                if self.consume(b'}') {
                    return Ok(());
                }
                loop {
                    self.string()?;
                    self.whitespace();
                    self.expect(b':')?;
                    self.whitespace();
                    self.value(depth + 1)?;
                    self.whitespace();
                    if self.consume(b'}') {
                        return Ok(());
                    }
                    self.expect(b',')?;
                    self.whitespace();
                }
            }
            Some(b'[') => {
                self.container_depth(depth)?;
                self.pos += 1;
                self.whitespace();
                if self.consume(b']') {
                    return Ok(());
                }
                loop {
                    self.value(depth + 1)?;
                    self.whitespace();
                    if self.consume(b']') {
                        return Ok(());
                    }
                    self.expect(b',')?;
                    self.whitespace();
                }
            }
            Some(b'"') => self.string().map(|_| ()),
            Some(b't') => self.literal(b"true"),
            Some(b'f') => self.literal(b"false"),
            Some(b'n') => self.literal(b"null"),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.error("expected a JSON value")),
        }
    }

    fn container_depth(&self, depth: usize) -> Result<(), String> {
        if depth >= MAX_DEPTH {
            Err(self.error("JSON nesting exceeds the 64-container inspection limit"))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_fresh_executable_with_fields_in_any_order() {
        let line = r#" {
            "executable": "/tmp/build target/debug/app",
            "target": {"kind": ["bin"], "name": "app"},
            "fresh": true,
            "reason": "compiler-artifact"
        } "#;
        assert_eq!(
            executable_from_message(line).unwrap(),
            Some(PathBuf::from("/tmp/build target/debug/app"))
        );
    }

    #[test]
    fn ignores_non_artifact_messages_and_missing_or_null_executables() {
        for line in [
            "warning: compiler message",
            "\tFinished `release` profile",
            "",
            "[not a Cargo object]",
            r#"{}"#,
            r#"{"reason":"build-finished","success":true}"#,
            r#"{"reason":"compiler-artifact","executable":null}"#,
            r#"{"reason":"compiler-artifact"}"#,
            r#"{"reason":"compiler-message","executable":"/wrong"}"#,
        ] {
            assert_eq!(executable_from_message(line).unwrap(), None, "{line}");
        }
    }

    #[test]
    fn never_extracts_nested_executables_or_filename_guesses() {
        for line in [
            r#"{"reason":"compiler-artifact","target":{"executable":"/wrong"}}"#,
            r#"{"reason":"compiler-artifact","filenames":["/wrong"]}"#,
            r#"{"wrapper":{"reason":"compiler-artifact","executable":"/wrong"}}"#,
            r#"{"reason":"compiler-artifact","data":[{"executable":"/wrong"}],"executable":null}"#,
        ] {
            assert_eq!(executable_from_message(line).unwrap(), None);
        }
    }

    #[test]
    fn decodes_unicode_surrogates_and_json_path_escapes() {
        let line = r#"{"reason":"compiler-artifact","executable":"C:\\build\\caf\u00e9-\uD83D\uDE80\t\"bin\"\/app"}"#;
        assert_eq!(
            executable_from_message(line).unwrap(),
            Some(PathBuf::from("C:\\build\\café-🚀\t\"bin\"/app"))
        );
        let line = r#"{"reason":"compiler-artifact","executable":"/tmp/日本語/🚀"}"#;
        assert_eq!(
            executable_from_message(line).unwrap(),
            Some(PathBuf::from("/tmp/日本語/🚀"))
        );
    }

    #[test]
    fn decodes_escaped_top_level_field_names() {
        let line = r#"{"rea\u0073on":"compiler-artifact","execu\u0074able":"/ok"}"#;
        assert_eq!(
            executable_from_message(line).unwrap(),
            Some(PathBuf::from("/ok"))
        );
    }

    #[test]
    fn validates_and_skips_nested_json_values() {
        let line = r#"{
            "reason":"compiler-artifact",
            "ignored":[null,true,false,0,-2,3.5,1e4,-0.12E-03,{},[],{"x":["a",{"b":2}]}],
            "executable":"/ok"
        }"#;
        assert_eq!(
            executable_from_message(line).unwrap(),
            Some(PathBuf::from("/ok"))
        );
    }

    #[test]
    fn rejects_malformed_json_even_inside_ignored_values() {
        for bad in [
            "01", "-", "1.", "1e", "+1", "NaN", "tru", "[1,]", "{\"a\":}",
        ] {
            let line = format!(
                "{{\"reason\":\"compiler-artifact\",\"ignored\":{bad},\"executable\":\"/wrong\"}}"
            );
            assert!(executable_from_message(&line).is_err(), "accepted {line}");
        }
        for line in [
            r#"{"reason":"compiler-artifact","executable":"/wrong",}"#,
            r#"{"reason":"compiler-artifact","executable":"/wrong"} trailing"#,
            r#"{"reason":"compiler-artifact" "executable":"/wrong"}"#,
            r#"{"reason":"compiler-artifact","executable":false}"#,
            r#"{"reason":"compiler-artifact","executable":""}"#,
        ] {
            assert!(executable_from_message(line).is_err(), "accepted {line}");
        }
    }

    #[test]
    fn rejects_malformed_unicode_and_unescaped_control_bytes() {
        for value in [
            r#""\uD800""#,
            r#""\uDC00""#,
            r#""\uD800\u0000""#,
            r#""\uD800x""#,
            r#""\u12""#,
            r#""\uWXYZ""#,
            r#""\q""#,
            "\"line\nbreak\"",
        ] {
            let line = format!("{{\"reason\":\"compiler-artifact\",\"executable\":{value}}}");
            assert!(executable_from_message(&line).is_err(), "accepted {line}");
        }
    }

    #[test]
    fn rejects_duplicate_selected_fields_including_escaped_duplicates() {
        for line in [
            r#"{"reason":"compiler-artifact","reason":"compiler-message"}"#,
            r#"{"reason":"compiler-artifact","executable":null,"executable":"/wrong"}"#,
            r#"{"reason":"compiler-artifact","executable":"/one","execu\u0074able":"/two"}"#,
        ] {
            assert!(executable_from_message(line)
                .unwrap_err()
                .contains("duplicate"));
        }
    }

    #[test]
    fn every_object_shaped_truncation_returns_an_error() {
        let line =
            r#"{"reason":"compiler-artifact","target":{"kind":["bin"]},"executable":"/tmp/hello"}"#;
        for end in 1..line.len() {
            assert!(
                executable_from_message(&line[..end]).is_err(),
                "accepted prefix {end}"
            );
        }
    }

    #[test]
    fn bounds_nesting_depth() {
        let line = format!(
            "{{\"ignored\":{}0{},\"reason\":\"compiler-artifact\",\"executable\":\"/wrong\"}}",
            "[".repeat(MAX_DEPTH),
            "]".repeat(MAX_DEPTH)
        );
        assert!(executable_from_message(&line)
            .unwrap_err()
            .contains("nesting"));
    }

    #[test]
    fn bounds_object_message_size() {
        let line = format!("{{\"ignored\":\"{}\"}}", "a".repeat(MAX_JSON_LINE_BYTES));
        assert!(executable_from_message(&line)
            .unwrap_err()
            .contains("16 MiB"));
    }
}
