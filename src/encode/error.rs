use std;
use std::io;
use trackable;

use Error;

#[derive(Debug)]
pub struct EncodeError<W> {
    pub writer: W,
    pub error: Error,
}
impl<W> EncodeError<W> {
    pub(crate) fn new(writer: W, error: Error) -> Self {
        EncodeError { writer, error }
    }
}
impl<W> std::fmt::Display for EncodeError<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
impl<W: std::fmt::Debug> std::error::Error for EncodeError<W> {
    fn description(&self) -> &str {
        self.error.description()
    }
    fn cause(&self) -> Option<&std::error::Error> {
        self.error.cause()
    }
}
impl<W> trackable::Trackable for EncodeError<W> {
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
impl<W> From<EncodeError<io::Take<W>>> for EncodeError<W> {
    fn from(f: EncodeError<io::Take<W>>) -> Self {
        EncodeError {
            writer: f.writer.into_inner(),
            error: f.error,
        }
    }
}
