use stakker::{Core, LogID};

/// Logging context
///
/// This encapsulates a [`stakker::LogID`] and a reference to
/// [`stakker::Core`].  It allows logging to a particular context or
/// span without having to carry around a reference to a particular
/// actor.  A reference to a [`LogCx`] can be used as the `[cx]`
/// argument to any of the logging macros.
///
/// [`LogCx`]: struct.LogCx.html
/// [`stakker::Core`]: ../stakker/struct.Core.html
/// [`stakker::LogID`]: ../stakker/type.LogID.html
pub struct LogCx<'a> {
    logid: LogID,
    core: &'a mut Core,
}

impl<'a> LogCx<'a> {
    /// Create directly from `LogID` and `Core` reference
    pub fn new(logid: LogID, core: &'a mut Core) -> Self {
        Self { logid, core }
    }

    /// Used by macros to obtain the `LogID`
    pub fn access_log_id(&self) -> LogID {
        self.logid
    }

    /// Used by macros to obtain the `Core` reference
    pub fn access_core(&mut self) -> &mut Core {
        self.core
    }
}
