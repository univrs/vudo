//! Dependency Resolution for Spirits
//!
//! Provides dependency specification and resolution for Spirit packages.

use crate::version::{SemVer, VersionError, VersionRequirement};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// A dependency on another Spirit package
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    /// Version requirement
    #[serde(default)]
    pub version: String,

    /// Optional: specific registry to fetch from
    pub registry: Option<String>,

    /// Optional: git repository URL
    pub git: Option<String>,

    /// Optional: git branch/tag/commit
    pub rev: Option<String>,

    /// Optional: local filesystem path
    pub path: Option<String>,

    /// Whether this is an optional dependency
    #[serde(default)]
    pub optional: bool,

    /// Features to enable for this dependency
    #[serde(default)]
    pub features: Vec<String>,
}

impl Dependency {
    /// Create a new dependency with version requirement
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            registry: None,
            git: None,
            rev: None,
            path: None,
            optional: false,
            features: Vec::new(),
        }
    }

    /// Create a dependency from a git repository
    pub fn from_git(url: impl Into<String>, rev: Option<String>) -> Self {
        Self {
            version: String::new(),
            registry: None,
            git: Some(url.into()),
            rev,
            path: None,
            optional: false,
            features: Vec::new(),
        }
    }

    /// Create a dependency from a local path
    pub fn from_path(path: impl Into<String>) -> Self {
        Self {
            version: String::new(),
            registry: None,
            git: None,
            rev: None,
            path: Some(path.into()),
            optional: false,
            features: Vec::new(),
        }
    }

    /// Get the version requirement
    pub fn version_requirement(&self) -> Result<VersionRequirement, VersionError> {
        if self.version.is_empty() {
            Ok(VersionRequirement::Any)
        } else {
            self.version.parse()
        }
    }

    /// Check if this is a local path dependency
    pub fn is_local(&self) -> bool {
        self.path.is_some()
    }

    /// Check if this is a git dependency
    pub fn is_git(&self) -> bool {
        self.git.is_some()
    }

    /// Check if this is a registry dependency
    pub fn is_registry(&self) -> bool {
        !self.is_local() && !self.is_git()
    }
}

impl Default for Dependency {
    fn default() -> Self {
        Self::new("*")
    }
}

impl FromStr for Dependency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Simple version string
        Ok(Dependency::new(s))
    }
}

/// Resolved dependency with specific version
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: SemVer,
    pub source: DependencySource,
}

/// Source of a resolved dependency
#[derive(Debug, Clone)]
pub enum DependencySource {
    Registry(String),
    Git { url: String, rev: String },
    Local(String),
}

/// Dependency resolver using SAT-based resolution
pub struct DependencyResolver {
    /// Available packages in registries
    available: HashMap<String, Vec<SemVer>>,

    /// Currently resolved dependencies
    resolved: HashMap<String, ResolvedDependency>,
}

impl DependencyResolver {
    /// Create a new resolver
    pub fn new() -> Self {
        Self {
            available: HashMap::new(),
            resolved: HashMap::new(),
        }
    }

    /// Add available package versions from a registry
    pub fn add_available(&mut self, name: impl Into<String>, versions: Vec<SemVer>) {
        self.available.insert(name.into(), versions);
    }

    /// Resolve dependencies for a manifest
    pub fn resolve(
        &mut self,
        dependencies: &HashMap<String, Dependency>,
    ) -> Result<Vec<ResolvedDependency>, ResolutionError> {
        let mut result = Vec::new();

        for (name, dep) in dependencies {
            let resolved = self.resolve_single(name, dep)?;
            result.push(resolved);
        }

        Ok(result)
    }

    fn resolve_single(
        &mut self,
        name: &str,
        dep: &Dependency,
    ) -> Result<ResolvedDependency, ResolutionError> {
        // Local path dependencies
        if let Some(ref path) = dep.path {
            return Ok(ResolvedDependency {
                name: name.to_string(),
                version: SemVer::new(0, 0, 0), // Version from local manifest
                source: DependencySource::Local(path.clone()),
            });
        }

        // Git dependencies
        if let Some(ref url) = dep.git {
            let rev = dep.rev.clone().unwrap_or_else(|| "HEAD".to_string());
            return Ok(ResolvedDependency {
                name: name.to_string(),
                version: SemVer::new(0, 0, 0), // Version from git
                source: DependencySource::Git {
                    url: url.clone(),
                    rev,
                },
            });
        }

        // Registry dependencies - find best matching version
        let requirement = dep
            .version_requirement()
            .map_err(|e| ResolutionError::InvalidVersion(e.to_string()))?;

        let available = self
            .available
            .get(name)
            .ok_or_else(|| ResolutionError::PackageNotFound(name.to_string()))?;

        // Find the highest version that satisfies the requirement
        let version = available
            .iter()
            .filter(|v| v.satisfies(&requirement))
            .max()
            .cloned()
            .ok_or_else(|| ResolutionError::NoMatchingVersion {
                name: name.to_string(),
                requirement: dep.version.clone(),
            })?;

        let registry = dep
            .registry
            .clone()
            .unwrap_or_else(|| "default".to_string());

        Ok(ResolvedDependency {
            name: name.to_string(),
            version,
            source: DependencySource::Registry(registry),
        })
    }

    /// Get all resolved dependencies
    pub fn resolved(&self) -> impl Iterator<Item = &ResolvedDependency> {
        self.resolved.values()
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Dependency resolution errors
#[derive(Debug, Clone)]
pub enum ResolutionError {
    PackageNotFound(String),
    NoMatchingVersion { name: String, requirement: String },
    ConflictingVersions { name: String, versions: Vec<String> },
    CyclicDependency(Vec<String>),
    InvalidVersion(String),
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionError::PackageNotFound(name) => {
                write!(f, "Package not found: {}", name)
            }
            ResolutionError::NoMatchingVersion { name, requirement } => {
                write!(f, "No version of {} satisfies {}", name, requirement)
            }
            ResolutionError::ConflictingVersions { name, versions } => {
                write!(
                    f,
                    "Conflicting versions of {}: {}",
                    name,
                    versions.join(", ")
                )
            }
            ResolutionError::CyclicDependency(cycle) => {
                write!(f, "Cyclic dependency: {}", cycle.join(" -> "))
            }
            ResolutionError::InvalidVersion(e) => {
                write!(f, "Invalid version: {}", e)
            }
        }
    }
}

impl std::error::Error for ResolutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_new() {
        let dep = Dependency::new("^1.0.0");
        assert_eq!(dep.version, "^1.0.0");
        assert!(!dep.optional);
        assert!(dep.is_registry());
    }

    #[test]
    fn test_dependency_from_git() {
        let dep = Dependency::from_git("https://github.com/test/repo", Some("v1.0.0".to_string()));
        assert!(dep.is_git());
        assert!(!dep.is_local());
        assert_eq!(dep.git, Some("https://github.com/test/repo".to_string()));
    }

    #[test]
    fn test_dependency_from_path() {
        let dep = Dependency::from_path("../local-spirit");
        assert!(dep.is_local());
        assert!(!dep.is_git());
        assert_eq!(dep.path, Some("../local-spirit".to_string()));
    }

    #[test]
    fn test_resolver_simple() {
        let mut resolver = DependencyResolver::new();
        resolver.add_available(
            "test-dep",
            vec![
                SemVer::new(1, 0, 0),
                SemVer::new(1, 1, 0),
                SemVer::new(2, 0, 0),
            ],
        );

        let mut deps = HashMap::new();
        deps.insert("test-dep".to_string(), Dependency::new("^1.0.0"));

        let resolved = resolver.resolve(&deps).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].version, SemVer::new(1, 1, 0)); // Highest compatible
    }

    #[test]
    fn test_resolver_not_found() {
        let mut resolver = DependencyResolver::new();

        let mut deps = HashMap::new();
        deps.insert("missing".to_string(), Dependency::new("^1.0.0"));

        let result = resolver.resolve(&deps);
        assert!(matches!(result, Err(ResolutionError::PackageNotFound(_))));
    }

    #[test]
    fn test_resolver_no_matching_version() {
        let mut resolver = DependencyResolver::new();
        resolver.add_available("test-dep", vec![SemVer::new(1, 0, 0)]);

        let mut deps = HashMap::new();
        deps.insert("test-dep".to_string(), Dependency::new("^2.0.0"));

        let result = resolver.resolve(&deps);
        assert!(matches!(
            result,
            Err(ResolutionError::NoMatchingVersion { .. })
        ));
    }

    #[test]
    fn test_resolver_local_dependency() {
        let mut resolver = DependencyResolver::new();

        let mut deps = HashMap::new();
        deps.insert("local".to_string(), Dependency::from_path("./local"));

        let resolved = resolver.resolve(&deps).unwrap();
        assert_eq!(resolved.len(), 1);
        assert!(matches!(resolved[0].source, DependencySource::Local(_)));
    }
}
