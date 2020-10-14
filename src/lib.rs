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
//! record (which can either be a plain identifier or a literal
//! string), followed by key-value pairs.
//!
//! Where the call is not being made from an actor context, `[core]`
//! may be passed instead of `[cx]`, which gives a `LogID` of zero.
//! It's possible to log against a specific actor or other `LogID`
//! source by using `[source, core]` instead of `[cx]`, which takes
//! the `LogID` from that source using a `source.access_log_id()`
//! call.
//!
//! The most general form of a key-value pair is `"key": expr`, but
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
//! [`Visitable`]: trait.Visitable.html
//! [`audit!`]: macro.audit.html
//! [`debug!`]: macro.debug.html
//! [`error!`]: macro.error.html
//! [`info!`]: macro.info.html
//! [`trace!`]: macro.trace.html
//! [`warn!`]: macro.warn.html

mod kvdisp;
mod macros;
mod visit;

pub use kvdisp::KvSingleLine;
pub use visit::Visitable;

// Re-export so that macros can access stakker::LogLevel
#[doc(hidden)]
pub use stakker;

// TODO: Add loggers that log to 'log' crate, 'slog' crate,
// 'tracing-core', etc

// TODO: Maybe add adapter to dump KVs or maybe whole record to JSON
// (single-line, packed)

#[cfg(test)]
mod test;
