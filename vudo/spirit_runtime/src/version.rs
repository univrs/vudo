//! Semantic Versioning for Spirits
//!
//! Implements SemVer 2.0.0 compatible versioning for Spirit packages.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Semantic version (major.minor.patch)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

impl SemVer {
    /// Create a new semantic version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
            build: None,
        }
    }

    /// Create with prerelease tag
    pub fn with_prerelease(mut self, prerelease: impl Into<String>) -> Self {
        self.prerelease = Some(prerelease.into());
        self
    }

    /// Create with build metadata
    pub fn with_build(mut self, build: impl Into<String>) -> Self {
        self.build = Some(build.into());
        self
    }

    /// Check if this is a stable release (no prerelease tag)
    pub fn is_stable(&self) -> bool {
        self.prerelease.is_none()
    }

    /// Check if major version is 0 (development phase)
    pub fn is_development(&self) -> bool {
        self.major == 0
    }

    /// Check if this version satisfies a version requirement
    pub fn satisfies(&self, requirement: &VersionRequirement) -> bool {
        match requirement {
            VersionRequirement::Exact(v) => self == v,
            VersionRequirement::GreaterThan(v) => self > v,
            VersionRequirement::GreaterOrEqual(v) => self >= v,
            VersionRequirement::LessThan(v) => self < v,
            VersionRequirement::LessOrEqual(v) => self <= v,
            VersionRequirement::Compatible(v) => self.is_compatible_with(v),
            VersionRequirement::Any => true,
        }
    }

    /// Check if this version is compatible with another (same major, >= minor)
    pub fn is_compatible_with(&self, other: &SemVer) -> bool {
        if self.major != other.major {
            return false;
        }
        if self.minor < other.minor {
            return false;
        }
        if self.minor == other.minor && self.patch < other.patch {
            return false;
        }
        true
    }

    /// Increment major version (resets minor and patch to 0)
    pub fn bump_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    /// Increment minor version (resets patch to 0)
    pub fn bump_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    /// Increment patch version
    pub fn bump_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
}

impl Default for SemVer {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.prerelease {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

impl FromStr for SemVer {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(VersionError::Empty);
        }

        // Split off build metadata first
        let (version_pre, build) = match s.split_once('+') {
            Some((v, b)) => (v, Some(b.to_string())),
            None => (s, None),
        };

        // Split off prerelease tag
        let (version, prerelease) = match version_pre.split_once('-') {
            Some((v, p)) => (v, Some(p.to_string())),
            None => (version_pre, None),
        };

        // Parse major.minor.patch
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::InvalidFormat(s.to_string()));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| VersionError::InvalidNumber(parts[0].to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| VersionError::InvalidNumber(parts[1].to_string()))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| VersionError::InvalidNumber(parts[2].to_string()))?;

        Ok(Self {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }
        // Prerelease versions have lower precedence than normal
        match (&self.prerelease, &other.prerelease) {
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
            (None, None) => Ordering::Equal,
        }
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Version requirement for dependency resolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionRequirement {
    /// Exact version match (=1.0.0)
    Exact(SemVer),
    /// Greater than (>1.0.0)
    GreaterThan(SemVer),
    /// Greater than or equal (>=1.0.0)
    GreaterOrEqual(SemVer),
    /// Less than (<1.0.0)
    LessThan(SemVer),
    /// Less than or equal (<=1.0.0)
    LessOrEqual(SemVer),
    /// Compatible version (^1.0.0 - same major, >= specified)
    Compatible(SemVer),
    /// Any version (*)
    Any,
}

impl FromStr for VersionRequirement {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s == "*" {
            return Ok(VersionRequirement::Any);
        }

        if let Some(rest) = s.strip_prefix(">=") {
            return Ok(VersionRequirement::GreaterOrEqual(rest.parse()?));
        }
        if let Some(rest) = s.strip_prefix("<=") {
            return Ok(VersionRequirement::LessOrEqual(rest.parse()?));
        }
        if let Some(rest) = s.strip_prefix('>') {
            return Ok(VersionRequirement::GreaterThan(rest.parse()?));
        }
        if let Some(rest) = s.strip_prefix('<') {
            return Ok(VersionRequirement::LessThan(rest.parse()?));
        }
        if let Some(rest) = s.strip_prefix('^') {
            return Ok(VersionRequirement::Compatible(rest.parse()?));
        }
        if let Some(rest) = s.strip_prefix('=') {
            return Ok(VersionRequirement::Exact(rest.parse()?));
        }

        // Default: compatible version
        Ok(VersionRequirement::Compatible(s.parse()?))
    }
}

/// Version parsing errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionError {
    Empty,
    InvalidFormat(String),
    InvalidNumber(String),
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionError::Empty => write!(f, "Version string is empty"),
            VersionError::InvalidFormat(s) => write!(f, "Invalid version format: {}", s),
            VersionError::InvalidNumber(s) => write!(f, "Invalid version number: {}", s),
        }
    }
}

impl std::error::Error for VersionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() {
        let v: SemVer = "1.2.3".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.prerelease.is_none());
        assert!(v.build.is_none());
    }

    #[test]
    fn test_semver_parse_with_prerelease() {
        let v: SemVer = "1.0.0-alpha.1".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.prerelease, Some("alpha.1".to_string()));
    }

    #[test]
    fn test_semver_parse_with_build() {
        let v: SemVer = "1.0.0+build.123".parse().unwrap();
        assert_eq!(v.build, Some("build.123".to_string()));
    }

    #[test]
    fn test_semver_parse_full() {
        let v: SemVer = "1.0.0-beta.2+build.456".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert_eq!(v.prerelease, Some("beta.2".to_string()));
        assert_eq!(v.build, Some("build.456".to_string()));
    }

    #[test]
    fn test_semver_display() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");

        let v = SemVer::new(1, 0, 0).with_prerelease("alpha");
        assert_eq!(v.to_string(), "1.0.0-alpha");

        let v = SemVer::new(1, 0, 0).with_build("build");
        assert_eq!(v.to_string(), "1.0.0+build");
    }

    #[test]
    fn test_semver_ordering() {
        let v1: SemVer = "1.0.0".parse().unwrap();
        let v2: SemVer = "1.0.1".parse().unwrap();
        let v3: SemVer = "1.1.0".parse().unwrap();
        let v4: SemVer = "2.0.0".parse().unwrap();

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
    }

    #[test]
    fn test_semver_prerelease_ordering() {
        let v1: SemVer = "1.0.0-alpha".parse().unwrap();
        let v2: SemVer = "1.0.0".parse().unwrap();

        assert!(v1 < v2); // Prerelease has lower precedence
    }

    #[test]
    fn test_version_requirement() {
        let v = SemVer::new(1, 2, 3);

        assert!(v.satisfies(&"^1.0.0".parse().unwrap()));
        assert!(v.satisfies(&">=1.0.0".parse().unwrap()));
        assert!(!v.satisfies(&"<1.0.0".parse().unwrap()));
        assert!(v.satisfies(&"*".parse().unwrap()));
    }

    #[test]
    fn test_compatible_versions() {
        let v1 = SemVer::new(1, 2, 3);
        let v2 = SemVer::new(1, 2, 0);
        let v3 = SemVer::new(1, 3, 0);
        let v4 = SemVer::new(2, 0, 0);

        assert!(v1.is_compatible_with(&v2)); // Same major, higher minor/patch
        assert!(!v1.is_compatible_with(&v3)); // Same major, lower minor
        assert!(!v1.is_compatible_with(&v4)); // Different major
    }

    #[test]
    fn test_bump_versions() {
        let v = SemVer::new(1, 2, 3);

        assert_eq!(v.bump_patch(), SemVer::new(1, 2, 4));
        assert_eq!(v.bump_minor(), SemVer::new(1, 3, 0));
        assert_eq!(v.bump_major(), SemVer::new(2, 0, 0));
    }
}
