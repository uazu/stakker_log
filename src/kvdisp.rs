use stakker::LogVisitor;
use std::fmt;
use std::fmt::Arguments;
use std::fmt::Write;

/// Single-line rendering of key-value pairs
///
/// When formatted with `"{}"`, this produces a human-readable
/// single-line rendering of the key-value pairs.  It is good for
/// debugging or simple display, or for input into human-readable
/// string-based processing.  All type information on key values is
/// lost.  Strings are shown without quotes if no characters need
/// quoting.  Simple `\XX` escaping is used for reserved ASCII
/// characters, where `XX` is two hex digits.  Anything higher than
/// ASCII is passed unchanged.  Arrays are enclosed in `[...]` and
/// maps are enclosed in `{...}`.
pub struct KvSingleLine<'a> {
    kvscan: &'a dyn Fn(&mut dyn LogVisitor),
    prefix: &'static str,
    suffix: &'static str,
}

impl<'a> KvSingleLine<'a> {
    /// Create a `KvSingleLine` ready to be formatted.  `prefix` and
    /// `suffix` are two strings which are output before and after the
    /// key-value pairs, but only if the list of key-value pairs is
    /// non-empty.
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

impl<'a> fmt::Display for KvSingleLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = Visitor::new(f, self.prefix);
        (self.kvscan)(&mut visitor);
        if visitor.error {
            Err(fmt::Error)
        } else if std::ptr::eq(visitor.prefix, self.prefix) {
            Ok(()) // Didn't output anything
        } else {
            f.write_str(self.suffix)
        }
    }
}

// Reserved characters outside quotes
#[inline]
fn is_reserved(ch: char) -> bool {
    ch <= ' '
        || ch == '"'
        || ch == '='
        || ch == '\\'
        || ch == '['
        || ch == ']'
        || ch == '{'
        || ch == '}'
}

// This has to be outside Visitor due to borrowing issues
#[inline]
fn push_str_val(f: &mut fmt::Formatter<'_>, val: &str) -> fmt::Result {
    if val.find(is_reserved).is_some() {
        f.write_char('"')?;
        for ch in val.chars() {
            if ch < ' ' || ch == '"' || ch == '\\' {
                write!(f, "\\{:02X}", ch as u8)?;
            } else {
                f.write_char(ch)?;
            }
        }
        f.write_char('"')?;
    } else {
        f.write_str(val)?;
    }
    Ok(())
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
    error: bool,
}

impl<'a, 'b> Visitor<'a, 'b> {
    fn new(fmt: &'a mut fmt::Formatter<'b>, prefix: &'static str) -> Self {
        Self {
            fmt,
            fmtbuf: String::new(),
            prefix,
            error: false,
        }
    }
    fn push_key(&mut self, key: Option<&str>, sep: Option<char>) {
        catch!(self, self.fmt.write_str(self.prefix));
        self.prefix = " ";
        if let Some(key) = key {
            if key.is_empty() {
                catch!(self, self.fmt.write_str("\\20"));
            } else {
                for ch in key.chars() {
                    if is_reserved(ch) {
                        catch!(self, write!(self.fmt, "\\{:02X}", ch as u8));
                    } else {
                        catch!(self, self.fmt.write_char(ch));
                    }
                }
            }
            if let Some(sep) = sep {
                catch!(self, self.fmt.write_char(sep));
            }
        }
    }
}

impl<'a, 'b> LogVisitor for Visitor<'a, 'b> {
    fn kv_u64(&mut self, key: Option<&str>, val: u64) {
        self.push_key(key, Some('='));
        catch!(self, write!(self.fmt, "{}", val));
    }
    fn kv_i64(&mut self, key: Option<&str>, val: i64) {
        self.push_key(key, Some('='));
        catch!(self, write!(self.fmt, "{}", val));
    }
    fn kv_f64(&mut self, key: Option<&str>, val: f64) {
        self.push_key(key, Some('='));
        catch!(self, write!(self.fmt, "{}", val));
    }
    fn kv_bool(&mut self, key: Option<&str>, val: bool) {
        self.push_key(key, Some('='));
        catch!(self, write!(self.fmt, "{}", val));
    }
    fn kv_null(&mut self, key: Option<&str>) {
        self.push_key(key, None);
    }
    fn kv_str(&mut self, key: Option<&str>, val: &str) {
        self.push_key(key, Some('='));
        catch!(self, push_str_val(self.fmt, val));
    }
    fn kv_fmt(&mut self, key: Option<&str>, val: &Arguments<'_>) {
        self.push_key(key, Some('='));
        if self.fmtbuf.capacity() == 0 {
            self.fmtbuf = String::with_capacity(1024);
        }
        self.fmtbuf.clear();
        catch!(self, write!(self.fmtbuf, "{}", val));
        catch!(self, push_str_val(self.fmt, &self.fmtbuf));
    }
    fn kv_map(&mut self, key: Option<&str>) {
        self.push_key(key, None);
        catch!(self, self.fmt.write_str("{"));
        self.prefix = "";
    }
    fn kv_mapend(&mut self, _: Option<&str>) {
        catch!(self, self.fmt.write_str("}"));
        self.prefix = " ";
    }
    fn kv_arr(&mut self, key: Option<&str>) {
        self.push_key(key, None);
        catch!(self, self.fmt.write_str("["));
        self.prefix = "";
    }
    fn kv_arrend(&mut self, _: Option<&str>) {
        catch!(self, self.fmt.write_str("]"));
        self.prefix = " ";
    }
}
