use std::fmt;

/// Errors that can occur when interacting with the Kubo FFI.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The provided path contained a null byte.
    InvalidPath,
    /// An operation failed in the Go layer.
    Go(String),
    /// The node handle is invalid or the node has been stopped.
    InvalidHandle,
    /// A string argument contained a null byte.
    InvalidString,
    /// The provided string is not a valid nostr URL.
    InvalidNostrUrl,
    /// The provided JSON is not a valid Nostr event.
    InvalidNostrEvent,
    /// No relay connection is available for the requested operation.
    NoRelayConnection,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidPath => write!(f, "path contains a null byte"),
            Error::Go(msg) => write!(f, "{msg}"),
            Error::InvalidHandle => write!(f, "invalid node handle"),
            Error::InvalidString => write!(f, "string contains a null byte"),
            Error::InvalidNostrUrl => write!(f, "invalid nostr URL"),
            Error::InvalidNostrEvent => write!(f, "invalid Nostr event JSON"),
            Error::NoRelayConnection => write!(f, "no relay connection available"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::ffi::NulError> for Error {
    fn from(_: std::ffi::NulError) -> Self {
        Error::InvalidString
    }
}
