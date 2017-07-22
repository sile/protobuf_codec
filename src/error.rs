use std;
use trackable;
use trackable::error::TrackableError;
use trackable::error::ErrorKind as TrackableErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Unsupported,
    Invalid,
    UnexpectedEos,
    Other,
}
impl TrackableErrorKind for ErrorKind {}

#[derive(Debug)]
pub struct Error<T> {
    pub stream: T,
    pub error: TrackableError<ErrorKind>,
}
impl<T> std::fmt::Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
impl<T: std::fmt::Debug> std::error::Error for Error<T> {
    fn description(&self) -> &str {
        self.error.description()
    }
    fn cause(&self) -> Option<&std::error::Error> {
        self.error.cause()
    }
}
impl<T> trackable::Trackable for Error<T> {
    type Event = trackable::error::Event;
    fn assign_tracking_number(&mut self) {
        self.error.assign_tracking_number();
    }
    fn tracking_number(&self) -> Option<trackable::TrackingNumber> {
        self.error.tracking_number()
    }
    fn enable_tracking(mut self) -> Self
    where
        Self: Sized,
    {
        self.error = self.error.enable_tracking();
        self
    }
    fn disable_tracking(mut self) -> Self
    where
        Self: Sized,
    {
        self.error = self.error.disable_tracking();
        self
    }
    fn history(&self) -> Option<&trackable::History<Self::Event>> {
        self.error.history()
    }
    fn history_mut(&mut self) -> Option<&mut trackable::History<Self::Event>> {
        self.error.history_mut()
    }
}
impl<T> From<Error<T>> for TrackableError<ErrorKind> {
    fn from(f: Error<T>) -> Self {
        f.error
    }
}
