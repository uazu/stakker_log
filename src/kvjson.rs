use stakker::LogVisitor;
use std::fmt;
use std::fmt::Arguments;
use std::fmt::Write;

/// JSON rendering of key-value pairs
///
/// When formatted with `"{}"`, this produces a single-line compact
/// JSON rendering of the key-value pairs.
pub struct KvToJson<'a> {
    kvscan: &'a dyn Fn(&mut dyn LogVisitor),
    prefix: &'static str,
    suffix: &'static str,
}

impl<'a> KvToJson<'a> {
    /// Create a `KvToJson` ready to be formatted.  The output will
    /// have commas between items, but not at the start or end.  A
    /// list of key-value pairs would normally be surrounded by `{`
    /// and `}` in JSON, but since this type may be used in other
    /// contexts, e.g. to extend a JSON object, only the plain
    /// key-value pairs are output.
    ///
    /// `prefix` and `suffix` are two strings which are output before
    /// and after the key-value pairs, but only if the list of
    /// key-value pairs is non-empty.  So, for example, passing `","`
    /// and `""` would be suitable for extending a larger JSON object
    /// that you're already building up.  Or alternatively, passing
    /// `",\"kv\":{"` and `"}"` would allow an optional `"kv"` item to
    /// be added to a larger JSON object, but only if there is
    /// key-value data.
    pub fn new(
        kvscan: &'a dyn Fn(&mut dyn LogVisitor),
        prefix: &'static str,
        suffix: &'static str,
    ) -> Self {
        Self {
            kvscan,
            prefix,
            suffix,
        }
    }
}

impl<'a> fmt::Display for KvToJson<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = Visitor::new(f, self.prefix);
        (self.kvscan)(&mut visitor);
        if visitor.error {
            Err(fmt::Error)
        } else if visitor.empty {
            Ok(()) // Didn't output anything
        } else {
            f.write_str(self.suffix)
        }
    }
}

/// JSON string quoting
#[inline]
fn push_str_literal(f: &mut fmt::Formatter<'_>, val: &str) -> fmt::Result {
    f.write_char('"')?;
    if val.find(|ch| ch < ' ' || ch == '"' || ch == '\\').is_some() {
        for ch in val.chars() {
            match ch {
                '"' | '\\' => {
                    f.write_char('\\')?;
                    f.write_char(ch)?;
                }
                '\u{0000}'..='\u{001F}' => write!(f, "\\u{:04X}", ch as u32)?,
                _ => f.write_char(ch)?,
            }
        }
    } else {
        f.write_str(val)?;
    }
    f.write_char('"')
}

// Catch error return and set error flag
macro_rules! catch {
    ($self:ident, $call:expr) => {{
        if $call.is_err() {
            $self.error = true;
        }
    }};
}

struct Visitor<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    fmtbuf: String,
    prefix: &'static str, // Whatever needs adding before the next item, or ""
    empty: bool,
    error: bool,
}

impl<'a, 'b> Visitor<'a, 'b> {
    fn new(fmt: &'a mut fmt::Formatter<'b>, prefix: &'static str) -> Self {
        Self {
            fmt,
            fmtbuf: String::new(),
            prefix,
            empty: true,
            error: false,
        }
    }
    fn push_key(&mut self, key: Option<&str>) {
        catch!(self, self.fmt.write_str(self.prefix));
        self.prefix = ",";
        self.empty = false;
        if let Some(key) = key {
            catch!(self, push_str_literal(self.fmt, key));
            catch!(self, self.fmt.write_char(':'));
        }
    }
}

impl<'a, 'b> LogVisitor for Visitor<'a, 'b> {
    fn kv_u64(&mut self, key: Option<&str>, val: u64) {
        self.push_key(key);
        catch!(self, write!(self.fmt, "{}", val));
    }
    fn kv_i64(&mut self, key: Option<&str>, val: i64) {
        self.push_key(key);
        catch!(self, write!(self.fmt, "{}", val));
    }
    fn kv_f64(&mut self, key: Option<&str>, val: f64) {
        self.push_key(key);
        catch!(self, write!(self.fmt, "{}", val));
    }
    fn kv_bool(&mut self, key: Option<&str>, val: bool) {
        self.push_key(key);
        catch!(self, write!(self.fmt, "{}", val));
    }
    fn kv_null(&mut self, key: Option<&str>) {
        self.push_key(key);
        catch!(self, self.fmt.write_str("null"));
    }
    fn kv_str(&mut self, key: Option<&str>, val: &str) {
        self.push_key(key);
        catch!(self, push_str_literal(self.fmt, val));
    }
    fn kv_fmt(&mut self, key: Option<&str>, val: &Arguments<'_>) {
        self.push_key(key);
        if self.fmtbuf.capacity() == 0 {
            self.fmtbuf = String::with_capacity(1024);
        }
        self.fmtbuf.clear();
        catch!(self, write!(self.fmtbuf, "{}", val));
        catch!(self, push_str_literal(self.fmt, &self.fmtbuf));
    }
    fn kv_map(&mut self, key: Option<&str>) {
        self.push_key(key);
        catch!(self, self.fmt.write_char('{'));
        self.prefix = "";
    }
    fn kv_mapend(&mut self, _: Option<&str>) {
        catch!(self, self.fmt.write_char('}'));
        self.prefix = ",";
    }
    fn kv_arr(&mut self, key: Option<&str>) {
        self.push_key(key);
        catch!(self, self.fmt.write_char('['));
        self.prefix = "";
    }
    fn kv_arrend(&mut self, _: Option<&str>) {
        catch!(self, self.fmt.write_char(']'));
        self.prefix = ",";
    }
}

#[cfg(test)]
mod test {
    use super::{KvToJson, LogVisitor};
    use std::fmt::Write;

    fn kvscan_empty(_lv: &mut dyn LogVisitor) {}

    fn kvscan_simple(lv: &mut dyn LogVisitor) {
        lv.kv_u64(Some("u64"), 123456789);
        lv.kv_str(Some("str"), "ABCDEFGHIJ");
    }

    fn kvscan_all(lv: &mut dyn LogVisitor) {
        lv.kv_u64(Some("u64"), 123456789);
        lv.kv_i64(Some("i64"), -123456789);
        lv.kv_f64(Some("f64"), 12345.6789);
        lv.kv_bool(Some("b0"), false);
        lv.kv_bool(Some("b1"), true);
        lv.kv_null(Some("null"));
        lv.kv_str(Some("str"), "ABCDEFGHIJ");
        lv.kv_str(Some("str_ctrl"), "ABC\tDEF");
        lv.kv_str(Some("str_quote"), "ABC\"DEF\"GHI");
        lv.kv_str(Some("str_bsl"), "ABC\\DEF\\GHI");
        lv.kv_fmt(Some("fmt"), &format_args!("{}{}{}", "ABC", 123, "DEF"));
        lv.kv_map(Some("map"));
        lv.kv_u64(Some("map_u64"), 987654321);
        lv.kv_str(Some("map_str"), "JIHGFEDCBA");
        lv.kv_map(Some("map_nested"));
        lv.kv_bool(Some("map_nested_bool"), false);
        lv.kv_mapend(Some("map_nested"));
        lv.kv_mapend(Some("map"));
        lv.kv_map(Some("map_empty"));
        lv.kv_mapend(Some("map_empty"));
        lv.kv_arr(Some("arr"));
        lv.kv_u64(None, 987654321);
        lv.kv_str(None, "JIHGFEDCBA");
        lv.kv_arr(None);
        lv.kv_bool(None, true);
        lv.kv_arrend(None);
        lv.kv_arrend(Some("arr"));
        lv.kv_arr(Some("arr_empty"));
        lv.kv_arrend(Some("arr_empty"));
    }

    fn append(
        s: &mut String,
        kvscan: &dyn Fn(&mut dyn LogVisitor),
        prefix: &'static str,
        suffix: &'static str,
    ) {
        write!(s, "{}", KvToJson::new(kvscan, prefix, suffix)).unwrap();
    }

    #[test]
    fn test() {
        // To verify JSON with `jq . -` (for example), run `cargo test
        // -- --nocapture` and paste in the printed JSON data
        let mut buf = "{\"dummy\":1".to_string();
        append(&mut buf, &kvscan_empty, ",", "");
        buf.push_str("}");
        println!("{}", buf);
        assert_eq!(buf, "{\"dummy\":1}");

        let mut buf = "{\"dummy\":1".to_string();
        append(&mut buf, &kvscan_simple, ",", "");
        buf.push_str("}");
        println!("{}", buf);
        assert_eq!(
            buf,
            "{\"dummy\":1,\"u64\":123456789,\"str\":\"ABCDEFGHIJ\"}"
        );

        let mut buf = "{\"dummy\":1".to_string();
        append(&mut buf, &kvscan_empty, ",\"kv\":{", "}");
        buf.push_str("}");
        println!("{}", buf);
        assert_eq!(buf, "{\"dummy\":1}");

        let mut buf = "{\"dummy\":1".to_string();
        append(&mut buf, &kvscan_simple, ",\"kv\":{", "}");
        buf.push_str("}");
        println!("{}", buf);
        assert_eq!(
            buf,
            "{\"dummy\":1,\"kv\":{\"u64\":123456789,\"str\":\"ABCDEFGHIJ\"}}"
        );

        let mut buf = "{\"dummy\":1".to_string();
        append(&mut buf, &kvscan_all, ",", "");
        buf.push_str("}");
        println!("{}", buf);
        assert_eq!(
            buf,
            "{\"dummy\":1,\"u64\":123456789,\"i64\":-123456789,\"f64\":12345.6789,\"b0\":false,\"b1\":true,\"null\":null,\"str\":\"ABCDEFGHIJ\",\"str_ctrl\":\"ABC\\u0009DEF\",\"str_quote\":\"ABC\\\"DEF\\\"GHI\",\"str_bsl\":\"ABC\\\\DEF\\\\GHI\",\"fmt\":\"ABC123DEF\",\"map\":{\"map_u64\":987654321,\"map_str\":\"JIHGFEDCBA\",\"map_nested\":{\"map_nested_bool\":false}},\"map_empty\":{},\"arr\":[987654321,\"JIHGFEDCBA\",[true]],\"arr_empty\":[]}"
        );
    }
}
