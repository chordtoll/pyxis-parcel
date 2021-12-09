use thiserror::Error;

/// An error enum for return from parcel methods that may fail
#[derive(Error, Debug)]
pub enum ParcelError {
    /// Cannot convert a string to/from unicode
    #[error("String conversion Error")]
    StringConversion,
    /// Requesting an object that doesn't exist
    #[error("Requested object does not exist")]
    Enoent,
    /// Trying to read from an object that's not a file
    #[error("Requested object not a file")]
    NotFile,
    /// Reading a parcel without a version field
    #[error("Missing version field")]
    NoVersion,
    /// Reading a parcel with a non-integer version
    #[error("Wrong version type")]
    VersionType,
    /// Trying to load a parcel created with a different format version
    #[error("Version Mismatch (expected {expected:?}, got {found:?})")]
    #[allow(missing_docs)]
    VersionMismatch { expected: u32, found: u32 },
}
