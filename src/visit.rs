use stakker::LogVisitor;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::fmt::Arguments;

/// This trait allows a `stakker::LogVisitor` to visit various
/// fundamental Rust types and collections.
///
/// It maps them to the fixed set of `kv_*` methods available on the
/// `stakker::LogVisitor`.  For types which must be formatted as a
/// string, construct a `std::fmt::Arguments` instance using
/// `format_args!` first, and then visit that.  For your own complex
/// types which need structured output, you should write your own
/// [`Visitable`] implementation.
///
/// For example:
///
/// ```notest
/// var1.visit("key1", visitor);     // Handle according to type
/// var2.visit("key2", visitor);
/// format_args!("{}", var3).visit("key3", visitor);   // Display formatting
/// format_args!("{:?}", var4).visit("key4", visitor); // Debug formatting
/// ```
///
/// [`Visitable`]: trait.Visitable.html
pub trait Visitable {
    fn visit(&self, key: Option<&str>, output: &mut dyn LogVisitor);
}

// String handling
impl Visitable for &str {
    #[inline]
    fn visit(&self, key: Option<&str>, output: &mut dyn LogVisitor) {
        output.kv_str(key, *self);
    }
}

impl Visitable for String {
    #[inline]
    fn visit(&self, key: Option<&str>, output: &mut dyn LogVisitor) {
        output.kv_str(key, self.as_str());
    }
}

// Null or unit handling
impl Visitable for () {
    #[inline]
    fn visit(&self, key: Option<&str>, output: &mut dyn LogVisitor) {
        output.kv_null(key);
    }
}

// Format arguments handling
impl<'a> Visitable for Arguments<'a> {
    #[inline]
    fn visit(&self, key: Option<&str>, output: &mut dyn LogVisitor) {
        output.kv_fmt(key, self);
    }
}

// Copy types and convertible types
macro_rules! visit_copy_as {
    ($fr:ty, $to:ty, $method:ident) => {
        impl $crate::Visitable for $fr {
            #[inline]
            fn visit(&self, key: Option<&str>, output: &mut dyn LogVisitor) {
                output.$method(key, *self as $to);
            }
        }
    };
}

visit_copy_as!(u8, u64, kv_u64);
visit_copy_as!(u16, u64, kv_u64);
visit_copy_as!(u32, u64, kv_u64);
visit_copy_as!(u64, u64, kv_u64);
visit_copy_as!(usize, u64, kv_u64);
visit_copy_as!(i8, i64, kv_i64);
visit_copy_as!(i16, i64, kv_i64);
visit_copy_as!(i32, i64, kv_i64);
visit_copy_as!(i64, i64, kv_i64);
visit_copy_as!(isize, i64, kv_i64);
visit_copy_as!(f32, f64, kv_f64);
visit_copy_as!(f64, f64, kv_f64);
visit_copy_as!(bool, bool, kv_bool);

// Types that we have to just format out as a string
macro_rules! visit_as_display {
    ($fr:ty) => {
        impl $crate::Visitable for $fr {
            #[inline]
            fn visit(&self, key: Option<&str>, output: &mut dyn LogVisitor) {
                output.kv_fmt(key, &format_args!("{}", *self));
            }
        }
    };
}

visit_as_display!(char);
visit_as_display!(u128);
visit_as_display!(i128);

// Array-like objects
macro_rules! visit_arr {
    ($t:ident, $fr:ty) => {
        impl<$t: Visitable> Visitable for $fr {
            #[inline]
            fn visit(&self, key: Option<&str>, output: &mut dyn LogVisitor) {
                output.kv_arr(key);
                for v in self.iter() {
                    v.visit(None, output);
                }
                output.kv_arrend(key);
            }
        }
    };
}

visit_arr!(T, [T]);
visit_arr!(T, Vec<T>);
visit_arr!(T, VecDeque<T>);
visit_arr!(T, LinkedList<T>);
visit_arr!(T, HashSet<T>);
visit_arr!(T, BTreeSet<T>);
visit_arr!(T, BinaryHeap<T>);

// Map-like objects
macro_rules! visit_map {
    ($fr:ident) => {
        impl<K: AsRef<str>, V: Visitable> Visitable for $fr<K, V> {
            #[inline]
            fn visit(&self, key: Option<&str>, output: &mut dyn LogVisitor) {
                output.kv_map(key);
                for (k, v) in self {
                    v.visit(Some(k.as_ref()), output);
                }
                output.kv_mapend(key);
            }
        }
    };
}

visit_map!(HashMap);
visit_map!(BTreeMap);
