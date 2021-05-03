use log::{debug, error, info, trace, warn};

pub trait Logger {
    fn error(&self, s: String) {
        error!("{}", s)
    }
    fn warn(&self, s: String) {
        warn!("{}", s)
    }
    fn info(&self, s: String) {
        info!("{}", s)
    }
    fn debug(&self, s: String) {
        debug!("{}", s)
    }
    fn trace(&self, s: String) {
        trace!("{}", s)
    }
}

pub struct LoggerImpl;

impl Logger for LoggerImpl {}
