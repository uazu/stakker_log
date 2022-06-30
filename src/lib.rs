//! Logging macros for **Stakker**
//!
//! There are five severity-based logging macros ([`trace!`],
//! [`debug!`], [`info!`], [`warn!`] and [`error!`]) and one macro
//! designed for logging records that have a fixed tag and no freeform
//! text ([`audit!`]).  Examples:
//!
//! ```ignore
//! error!([cx], addr: %src_addr, port, "Failed to connect: {}", err);
//! audit!([cx], TcpConnectFailure, addr: %src_addr, port);
//! ```
//!
//! The severity-based macros like [`error!`] all work the same way.
//! The `[cx]` comes first, followed by an optional target
//! specification (`target: "target-name"`), followed by optional
//! key-value pairs, followed by a format-string and its arguments.
//!
//! For [`audit!`], `[cx]` comes first, followed by a tag for the
//! record, followed by key-value pairs.  The tag will normally be a
//! plain identifier, but it could also be a literal string or an
//! expression in parentheses which will be formatted to generate the
//! tag.
//!
//! `[cx]` can refer to either an actor context (`stakker::Cx`) or a
//! [`LogCx`].  Where the call is not being made from a context that
//! provides a `LogID`, `[core]` may be passed instead of `[cx]`,
//! which gives a `LogID` of zero.  It's possible to log against a
//! specific actor or other `LogID` source by using `[source, core]`
//! instead of `[cx]`, which takes the `LogID` from that source using
//! a `source.access_log_id()` call.  (In general the `[a]` form must
//! support `a.access_log_id()` and `a.access_core()`, and the `[a,b]`
//! form must support `a.access_log_id()` and `b.access_core()`.)
//!
//! For key-value pairs, the most general form is `"key": expr`, but
//! there are a number of shortcuts as follows:
//!
//! Shortcut | Equivalent
//! --- | ---
//! `size: file_size` | `"size": file_size`
//! `size` | `"size": size`
//! `packet.size` | `"size": packet.size`
//! `tcp.packet.size` | `"size": tcp.packet.size`
//! `%src_addr` | `"src_addr": format_args!("{}", src_addr)`
//! `src_addr: %addr` | `"src_addr": format_args!("{}", addr)`
//! `?stream` | `"stream": format_args!("{:?}", stream)`
//! `stream: ?input_stream` | `"stream": format_args!("{:?}", input_stream)`
//!
//! Conversion of values is determined by implementation of the
//! [`Visitable`] trait.  All Rust primitives and standard collections
//! are supported by [`Visitable`].  For your own types, there are two
//! possibilities: either implement `Display` or `Debug` and use `%`
//! or `?`, or else implement [`Visitable`] directly.  The advantage
//! of implementing [`Visitable`] is that in addition to mapping to a
//! primitive instead of a string, you could also output a structured
//! value such as an array or map to represent your type.
//!
//! # Logging output
//!
//! You can write you own code which accepts a `&dyn Fn(&mut dyn
//! LogVisitor)`, and calls it to receive all the logging data.  There
//! are also provided types for JSON output ([`KvToJson`]) and simple
//! human-readable output ([`KvSingleLine`]).
//!
//! [`KvSingleLine`]: struct.KvSingleLine.html
//! [`KvToJson`]: struct.KvToJson.html
//! [`LogCx`]: struct.LogCx.html
//! [`Visitable`]: trait.Visitable.html
//! [`audit!`]: macro.audit.html
//! [`debug!`]: macro.debug.html
//! [`error!`]: macro.error.html
//! [`info!`]: macro.info.html
//! [`trace!`]: macro.trace.html
//! [`warn!`]: macro.warn.html

mod kvdisp;
mod kvjson;
mod logcx;
mod macros;
mod visit;

pub use kvdisp::KvSingleLine;
pub use kvjson::KvToJson;
pub use logcx::LogCx;
pub use visit::Visitable;

// Re-export so that macros can access stakker::LogLevel
#[doc(hidden)]
pub use stakker;

// TODO: Add loggers that log to 'log' crate, 'slog' crate,
// 'tracing-core', etc

#[cfg(test)]
mod test;
