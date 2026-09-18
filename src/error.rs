use std::fmt;

use crate::path::TagPathError;

/// Errors returned by tag-tree operations.
///
/// This enum is non-exhaustive so callers remain source-compatible when new
/// error cases are added. Include a wildcard arm when matching it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
    CannotMoveIntoDescendant {
        /// The subtree path being moved.
        from: String,
        /// The rejected destination path.
        to: String,
    },
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

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPath(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TagPathError> for Error {
    fn from(error: TagPathError) -> Self {
        Self::InvalidPath(error)
    }
}

/// The result type used by public mutating and querying operations.
pub type Result<T> = std::result::Result<T, Error>;
