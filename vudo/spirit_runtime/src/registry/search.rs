//! Search utilities for Spirit registry
//!
//! Provides a fluent API for building search queries and utilities
//! for sorting and filtering search results.

use crate::manifest::Capability;

use super::types::{SpiritQuery, SpiritSearchResult};

// ═══════════════════════════════════════════════════════════════════════════
// QUERY BUILDER
// ═══════════════════════════════════════════════════════════════════════════

/// Fluent builder for constructing Spirit search queries
///
/// # Example
///
/// ```ignore
/// use spirit_runtime::registry::QueryBuilder;
///
/// let query = QueryBuilder::new()
///     .name("hello")
///     .author("vudo-team")
///     .capability("SensorTime")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    query: SpiritQuery,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by name pattern (partial match)
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.query.name = Some(name.into());
        self
    }

    /// Filter by author
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.query.author = Some(author.into());
        self
    }

    /// Require a capability (can be called multiple times)
    pub fn capability(mut self, cap: impl Into<String>) -> Self {
        self.query.capabilities.push(cap.into());
        self
    }

    /// Require a typed Capability
    pub fn with_capability(mut self, cap: Capability) -> Self {
        self.query.capabilities.push(format!("{:?}", cap));
        self
    }

    /// Filter by version constraint
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.query.version = Some(version.into());
        self
    }

    /// Build the query
    pub fn build(self) -> SpiritQuery {
        self.query
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SORT OPTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Sorting options for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    /// Sort by name alphabetically (default)
    #[default]
    Name,
    /// Sort by version (semantic versioning)
    Version,
    /// Sort by author name
    Author,
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    /// Ascending order (default)
    #[default]
    Ascending,
    /// Descending order
    Descending,
}

// ═══════════════════════════════════════════════════════════════════════════
// SEARCH UTILITIES
// ═══════════════════════════════════════════════════════════════════════════

/// Sort search results
pub fn sort_results(results: &mut [SpiritSearchResult], sort_by: SortBy, order: SortOrder) {
    results.sort_by(|a, b| {
        let cmp = match sort_by {
            SortBy::Name => a.name.cmp(&b.name),
            SortBy::Version => compare_versions(&a.version, &b.version),
            SortBy::Author => {
                let author_a = &a.manifest.author;
                let author_b = &b.manifest.author;
                author_a.cmp(author_b)
            }
        };

        match order {
            SortOrder::Ascending => cmp,
            SortOrder::Descending => cmp.reverse(),
        }
    });
}

/// Compare two semantic version strings
///
/// Returns Ordering based on semantic version comparison.
/// Falls back to string comparison if versions aren't valid semver.
pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_version = |s: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    };

    match (parse_version(a), parse_version(b)) {
        (Some((a1, a2, a3)), Some((b1, b2, b3))) => (a1, a2, a3).cmp(&(b1, b2, b3)),
        _ => a.cmp(b),
    }
}

/// Filter results by capability
pub fn filter_by_capability(
    results: Vec<SpiritSearchResult>,
    capability: &str,
) -> Vec<SpiritSearchResult> {
    results
        .into_iter()
        .filter(|r| {
            r.manifest
                .capabilities
                .iter()
                .any(|c| format!("{:?}", c).contains(capability))
        })
        .collect()
}

/// Match a name pattern against a spirit name
///
/// Supports:
/// - Exact match
/// - Prefix match (pattern*)
/// - Suffix match (*pattern)
/// - Contains match (*pattern*)
pub fn matches_name_pattern(name: &str, pattern: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let name = name.to_lowercase();

    if pattern.starts_with('*') && pattern.ends_with('*') {
        // Contains match
        let inner = &pattern[1..pattern.len() - 1];
        name.contains(inner)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        // Suffix match
        name.ends_with(suffix)
    } else if pattern.ends_with('*') {
        // Prefix match
        let prefix = &pattern[..pattern.len() - 1];
        name.starts_with(prefix)
    } else {
        // Exact or contains match
        name == pattern || name.contains(&pattern)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new()
            .name("hello")
            .author("test-author")
            .capability("SensorTime")
            .version("0.1.0")
            .build();

        assert_eq!(query.name, Some("hello".to_string()));
        assert_eq!(query.author, Some("test-author".to_string()));
        assert_eq!(query.capabilities, vec!["SensorTime"]);
        assert_eq!(query.version, Some("0.1.0".to_string()));
    }

    #[test]
    fn test_query_builder_multiple_capabilities() {
        let query = QueryBuilder::new()
            .capability("NetworkListen")
            .capability("StorageRead")
            .capability("SensorTime")
            .build();

        assert_eq!(query.capabilities.len(), 3);
    }

    #[test]
    fn test_compare_versions() {
        use std::cmp::Ordering;

        assert_eq!(compare_versions("0.1.0", "0.1.0"), Ordering::Equal);
        assert_eq!(compare_versions("0.1.0", "0.2.0"), Ordering::Less);
        assert_eq!(compare_versions("0.2.0", "0.1.0"), Ordering::Greater);
        assert_eq!(compare_versions("1.0.0", "0.9.9"), Ordering::Greater);
        assert_eq!(compare_versions("0.10.0", "0.9.0"), Ordering::Greater);
    }

    #[test]
    fn test_matches_name_pattern() {
        // Exact match
        assert!(matches_name_pattern("hello-world", "hello-world"));

        // Contains match
        assert!(matches_name_pattern("hello-world", "world"));
        assert!(matches_name_pattern("hello-world", "*world*"));

        // Prefix match
        assert!(matches_name_pattern("hello-world", "hello*"));
        assert!(!matches_name_pattern("goodbye-world", "hello*"));

        // Suffix match
        assert!(matches_name_pattern("hello-world", "*world"));
        assert!(!matches_name_pattern("hello-universe", "*world"));

        // Case insensitive
        assert!(matches_name_pattern("Hello-World", "hello*"));
    }

    #[test]
    fn test_sort_order() {
        assert_eq!(SortBy::default(), SortBy::Name);
        assert_eq!(SortOrder::default(), SortOrder::Ascending);
    }
}
