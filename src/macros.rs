// TODO: Add #[cfg] options to disable levels completely, eliminating
// all the code associated with logging those levels completely from
// the executable

// TODO: Switch to proc macros to allow us to automatically access
// `cx` without mentioning it explicitly.

/// Log an error with context info
///
/// See [top-level docs](index.html) for details.
#[macro_export]
macro_rules! error {
    ( $($x:tt)+ ) => {{
        $crate::log!(Error $($x)+);
    }}
}

/// Log a warning with context info
///
/// See [top-level docs](index.html) for details.
#[macro_export]
macro_rules! warn {
    ( $($x:tt)+ ) => {{
        $crate::log!(Warn $($x)+);
    }}
}

/// Log information with context info
///
/// See [top-level docs](index.html) for details.
#[macro_export]
macro_rules! info {
    ( $($x:tt)+ ) => {{
        $crate::log!(Info $($x)+);
    }}
}

/// Log debugging with context info
///
/// See [top-level docs](index.html) for details.
#[macro_export]
macro_rules! debug {
    ( $($x:tt)+ ) => {{
        $crate::log!(Debug $($x)+);
    }}
}

/// Log tracing with context info
///
/// See [top-level docs](index.html) for details.
#[macro_export]
macro_rules! trace {
    ( $($x:tt)+ ) => {{
        $crate::log!(Trace $($x)+);
    }}
}

/// Log an audit record
///
/// See [top-level docs](index.html) for details.
#[macro_export]
macro_rules! audit {
    ( [$($cx:tt)+], $tag:ident $(, $($tail:tt)+)? ) => {{
        $crate::log!(Audit [$($cx)+] $(, $($tail)+)? , "{}", ::std::stringify!($tag));
    }};
    ( [$($cx:tt)+], $tag:literal $(, $($tail:tt)+)? ) => {{
        $crate::log!(Audit [$($cx)+] $(, $($tail)+)? , "{}", $tag);
    }};
    ( [$($cx:tt)+], ($tag:expr) $(, $($tail:tt)+)? ) => {{
        $crate::log!(Audit [$($cx)+] $(, $($tail)+)? , "{}", $tag);
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_key_string {
    ($key:ident) => {
        ::std::stringify!($key)
    };
    ($base:ident . $($tail:tt)+) => {
        $crate::log_key_string!($($tail)+)
    };
}

/// Internal macro which handles translation to a log call
//
// Called with: `(level [cx] kv-args fmt fmt-args)`
#[macro_export]
macro_rules! log {
    // Initial part
    ($level:ident $fmt:literal $($tail:tt)*) => {{
        ::std::compile_error!("Stakker logging macros need `[cx]` or `[core]` or `[actor, core]` as first argument");
    }};
    ($level:ident [$cx:expr] $(, $($tail:tt)+)?) => {{
        $crate::log!($level [$cx, $cx] $(, $($tail)+)?)
    }};
    ($level:ident [$src:expr, $core:expr], target: $target:literal $(, $($tail:tt)+)?) => {{
        $crate::log!([$src.access_log_id(), $core, $level, $target] $($($tail)+)?)
    }};
    ($level:ident [$src:expr, $core:expr] $(, $($tail:tt)+)?) => {{
        $crate::log!([$src.access_log_id(), $core, $level, ""] $($($tail)+)?)
    }};
    ($level:ident $($tail:tt)*) => {{
        ::std::compile_error!("Stakker logging macros need `[cx]` or `[core]` or `[actor, core]` as first argument");
    }};
    // Primitive values (no % or ?)
    ([$($a:tt)*] $key1:ident $(. $key2:ident)*  $(, $($tail:tt)*)?) => {
        $crate::log!([$($a)* ($crate::log_key_string!($key1$(.$key2)*), $key1$(.$key2)*)] $($($tail)*)?)
    };
    ([$($a:tt)*] $key:ident : $value:expr $(, $($tail:tt)*)?) => {
        $crate::log!([$($a)* (::std::stringify!($key), $value)] $($($tail)*)?)
    };
    ([$($a:tt)*] $key:literal : $value:expr $(, $($tail:tt)*)?) => {
        $crate::log!([$($a)* ($key, $value)] $($($tail)*)?)
    };
    // Display-formatted values (with %)
    ([$($a:tt)*] % $key1:ident $(. $key2:ident)* $(, $($tail:tt)*)?) => {{
        let v = &($key1$(.$key2)*); // Do borrow outside of closure
        $crate::log!([$($a)* ($crate::log_key_string!($key1$(.$key2)*), format_args!("{}", v))] $($($tail)*)?)
    }};
    ([$($a:tt)*] $key:ident : % $value:expr $(, $($tail:tt)*)?) => {{
        let v = &$value; // Do borrow outside of closure
        $crate::log!([$($a)* (::std::stringify!($key), format_args!("{}", v))] $($($tail)*)?)
    }};
    ([$($a:tt)*] $key:literal : % $value:expr $(, $($tail:tt)*)?) => {{
        let v = &$value; // Do borrow outside of closure
        $crate::log!([$($a)* ($key, format_args!("{}", v))] $($($tail)*)?)
    }};
    // Debug-formatted values (with ?)
    ([$($a:tt)*] ? $key1:ident $(. $key2:ident)* $(, $($tail:tt)*)?) => {{
        let v = &($key1$(.$key2)*); // Do borrow outside of closure
        $crate::log!([$($a)* ($crate::log_key_string!($key1$(.$key2)*), format_args!("{:?}", v))] $($($tail)*)?)
    }};
    ([$($a:tt)*] $key:ident : ? $value:expr $(, $($tail:tt)*)?) => {{
        let v = &$value; // Do borrow outside of closure
        $crate::log!([$($a)* (::std::stringify!($key), format_args!("{:?}", v))] $($($tail)*)?)
    }};
    ([$($a:tt)*] $key:literal : ? $value:expr $(, $($tail:tt)*)?) => {
        let v = &$value; // Do borrow outside of closure
        $crate::log!([$($a)* ($key, format_args!("{:?}", v))] $($($tail)*)?)
    };
    // Final output
    ([$logid:expr, $core:expr, $level:ident, $target:literal $( ($key:expr, $val:expr) )*] $fmt:literal $(, $($tail:tt)*)?) => {{
        use $crate::Visitable;
        let id = $logid;
        let core = $core.access_core();
        core.log(
            id,
            $crate::stakker::LogLevel::$level,
            $target,
            ::std::format_args!( $fmt $(, $($tail)*)? ),
            |output| {
                $( $val.visit(Some($key), output); )*
            });
    }};
}
