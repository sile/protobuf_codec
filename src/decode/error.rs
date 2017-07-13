use std;
use std::io;
use trackable;

use Error;

#[derive(Debug)]
pub struct DecodeError<R> {
    pub reader: R,
    pub error: Error,
}
impl<R> DecodeError<R> {
    pub(crate) fn new(reader: R, error: Error) -> Self {
        DecodeError { reader, error }
    }
}
impl<R> std::fmt::Display for DecodeError<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
impl<R: std::fmt::Debug> std::error::Error for DecodeError<R> {
    fn description(&self) -> &str {
        self.error.description()
    }
    fn cause(&self) -> Option<&std::error::Error> {
        self.error.cause()
    }
}
impl<R> trackable::Trackable for DecodeError<R> {
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
impl<R> From<DecodeError<io::Take<R>>> for DecodeError<R> {
    fn from(f: DecodeError<io::Take<R>>) -> Self {
        DecodeError {
            reader: f.reader.into_inner(),
            error: f.error,
        }
    }
}
