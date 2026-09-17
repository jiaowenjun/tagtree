//! Helpers for slash-separated tag paths.

use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagPathError {
    value: String,
}

impl TagPathError {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for TagPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid tag path: {}", self.value)
    }
}

impl std::error::Error for TagPathError {}

pub fn normalize_path(value: impl AsRef<str>) -> Result<Option<String>, TagPathError> {
    let value = value.as_ref().trim();
    if value.is_empty() {
        return Ok(None);
    }

    validate_path(value)?;
    Ok(Some(value.to_string()))
}

pub fn normalize_paths<I, S>(values: I) -> Result<Vec<String>, TagPathError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();

    for value in values {
        if let Some(path) = normalize_path(value)? {
            if seen.insert(path.clone()) {
                normalized.push(path);
            }
        }
    }

    Ok(normalized)
}

pub fn validate_path(value: &str) -> Result<(), TagPathError> {
    if value.is_empty() {
        return Ok(());
    }

    let segments = value.split('/').collect::<Vec<_>>();
    if value.starts_with('/')
        || value.ends_with('/')
        || value.contains("//")
        || segments.iter().any(|segment| segment.trim().is_empty())
    {
        return Err(TagPathError::new(value));
    }

    Ok(())
}

/// Returns whether `candidate` equals `ancestor` or lies below it.
///
/// The empty path is the root and contains every path.
pub fn is_within(candidate: &str, ancestor: &str) -> bool {
    if ancestor.is_empty() {
        return true;
    }
    candidate == ancestor
        || candidate
            .strip_prefix(ancestor)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_path_skips_blank_values() {
        assert_eq!(normalize_path("  ").unwrap(), None);
    }

    #[test]
    fn validate_path_rejects_empty_segments() {
        assert_eq!(validate_path("a//b"), Err(TagPathError::new("a//b")));
    }
}
