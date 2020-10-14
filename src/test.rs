use stakker::*;
use std::cell::Cell;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::rc::Rc;
use std::time::Instant;

use crate::{error, KvSingleLine, Visitable};

// TODO: Need tests of all the different shortcuts
// TODO: Need test of audit!

struct MyType;
impl Visitable for MyType {
    fn visit(&self, key: Option<&str>, out: &mut dyn LogVisitor) {
        out.kv_map(key);
        out.kv_u64(Some("a"), 135);
        out.kv_null(Some("b"));
        out.kv_arr(Some("c"));
        out.kv_arrend(Some("c"));
        out.kv_mapend(key);
    }
}

#[test]
fn error_formatting() {
    let mut stakker = Stakker::new(Instant::now());
    let s = &mut stakker;
    let out = Rc::new(Cell::new(String::new()));
    let out2 = out.clone();

    s.set_logger(
        LogFilter::all(&[LogLevel::Trace, LogLevel::Audit, LogLevel::Open]),
        move |_, r| {
            out2.set(format!(
                "{} #{} {} {}",
                r.level,
                r.id,
                r.fmt,
                KvSingleLine::new(r.kvscan, "{", "}")
            ));
        },
    );

    let a = "TEST";
    let b = 1.234_f64;
    let c = 1234_i32;
    let d = -1234_i16;
    let e = 4321_u64;
    let f = vec!["abc", "def"];
    let g = Ipv4Addr::LOCALHOST;
    let h = ();
    let mut i = HashMap::new();
    i.insert("a", "cat");
    i.insert("b", "dog");
    let j = "This is a test";
    let k = MyType;
    error!([s], a, b, c, d, e, f, %g, h, i, j, k, "Test");
    let o = out.take();
    // Hashmap is unordered, so there are two possibilities
    match o.as_str() {
        "ERROR #0 Test {a=TEST b=1.234 c=1234 d=-1234 e=4321 f[abc def] \
         g=127.0.0.1 h i{a=cat b=dog} j=\"This is a test\" k{a=135 b c[]}}" => (),
        "ERROR #0 Test {a=TEST b=1.234 c=1234 d=-1234 e=4321 f[abc def] \
         g=127.0.0.1 h i{b=dog a=cat} j=\"This is a test\" k{a=135 b c[]}}" => (),
        _ => panic!("Unexpected output: {}", o),
    }
}
