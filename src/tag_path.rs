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

pub type TagPathResult<T> = Result<T, TagPathError>;

pub fn normalize_tag(value: impl AsRef<str>) -> TagPathResult<Option<String>> {
    let value = value.as_ref().trim();
    if value.is_empty() {
        return Ok(None);
    }

    validate_tag_path(value)?;
    Ok(Some(value.to_string()))
}

pub fn normalize_tags<I, S>(values: I) -> TagPathResult<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();

    for value in values {
        if let Some(tag) = normalize_tag(value)? {
            if seen.insert(tag.clone()) {
                normalized.push(tag);
            }
        }
    }

    Ok(normalized)
}

pub fn validate_tag_path(value: &str) -> TagPathResult<()> {
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

pub fn is_descendant_or_self(candidate: &str, ancestor: &str) -> bool {
    candidate == ancestor
        || candidate
            .strip_prefix(ancestor)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_tag_skips_blank_values() {
        assert_eq!(normalize_tag("  ").unwrap(), None);
    }

    #[test]
    fn validate_tag_path_rejects_empty_segments() {
        assert_eq!(validate_tag_path("a//b"), Err(TagPathError::new("a//b")));
    }
}
