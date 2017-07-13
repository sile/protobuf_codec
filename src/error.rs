use trackable::error::TrackableError;
use trackable::error::ErrorKind as TrackableErrorKind;

use decode::DecodeError;

#[derive(Debug, Clone)]
pub struct Error(TrackableError<ErrorKind>);
derive_traits_for_trackable_error_newtype!(Error, ErrorKind);
impl<R> From<DecodeError<R>> for Error {
    fn from(f: DecodeError<R>) -> Self {
        f.error
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Unsupported,
    Invalid,
    UnexpectedEos,
    Other,
}
impl TrackableErrorKind for ErrorKind {}
