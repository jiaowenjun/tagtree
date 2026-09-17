use std::fmt;

use crate::path::TagPathError;

/// Errors returned by the public tag-tree and path-tree operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A supplied path does not follow the slash-separated path rules.
    InvalidPath(TagPathError),
    /// The requested source or query path does not exist.
    PathNotFound(String),
    /// The root node cannot be moved into another path.
    CannotMoveRoot,
    /// The root path cannot be removed as a subtree.
    CannotRemoveRoot,
    /// A path cannot be moved into one of its own descendants.
    CannotMoveIntoDescendant { from: String, to: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(error) => error.fmt(f),
            Self::PathNotFound(path) => write!(f, "path not found: {path}"),
            Self::CannotMoveRoot => write!(f, "cannot move the root path"),
            Self::CannotRemoveRoot => write!(f, "cannot remove the root path"),
            Self::CannotMoveIntoDescendant { from, to } => {
                write!(f, "cannot move {from} into its descendant {to}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<TagPathError> for Error {
    fn from(error: TagPathError) -> Self {
        Self::InvalidPath(error)
    }
}

/// The result type used by public mutating and querying operations.
pub type Result<T> = std::result::Result<T, Error>;
