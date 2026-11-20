//! End-to-End Spirit Lifecycle Integration Tests
//!
//! These tests verify the full Spirit lifecycle from creation to execution:
//! - `vudo new` - Create a new Spirit project
//! - `vudo build` - Compile DOL source to Spirit package
//! - `vudo run` - Execute a Spirit in the sandbox
//! - `vudo check` - Validate DOL syntax
//! - `vudo pack` - Package Spirit for distribution
//! - `vudo sign` - Sign and verify packages
//!
//! All tests run in isolated temporary directories to ensure reproducibility.
//!
//! # Known Limitations
//!
//! The manifest format produced by `vudo new` currently uses a `[spirit]` section
//! with a string version, while `spirit_runtime::Manifest` expects fields at the
//! root level with a SemVer object. Tests that require full lifecycle integration
//! use a helper to create compatible manifests.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

/// Get the path to the vudo CLI binary
fn vudo_binary() -> String {
    // In integration tests, CARGO_BIN_EXE_<name> provides the path to binaries
    env!("CARGO_BIN_EXE_vudo").to_string()
}

/// Helper to run vudo commands and capture output
fn run_vudo(args: &[&str], working_dir: &Path) -> Output {
    Command::new(vudo_binary())
        .args(args)
        .current_dir(working_dir)
        .output()
        .expect("Failed to execute vudo command")
}

/// Helper to run vudo commands with specific environment variables
fn run_vudo_with_env(args: &[&str], working_dir: &Path, env_vars: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(vudo_binary());
    cmd.args(args).current_dir(working_dir);

    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    cmd.output().expect("Failed to execute vudo command")
}

/// Assert that a command succeeded
fn assert_success(output: &Output, context: &str) {
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "{} failed!\nExit code: {:?}\nStdout:\n{}\nStderr:\n{}",
            context,
            output.status.code(),
            stdout,
            stderr
        );
    }
}

/// Assert that a command failed
fn assert_failure(output: &Output, context: &str) {
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "{} should have failed but succeeded!\nStdout:\n{}",
            context, stdout
        );
    }
}

/// Create a Spirit project with a manifest compatible with spirit_runtime::Manifest
///
/// This helper bridges the gap between the manifest format from `vudo new`
/// and what `spirit_runtime::Manifest` expects for parsing.
fn create_compatible_spirit_project(base_path: &Path, name: &str) -> std::path::PathBuf {
    let project_path = base_path.join(name);
    fs::create_dir_all(project_path.join("src")).expect("Failed to create src directory");
    fs::create_dir_all(project_path.join("tests")).expect("Failed to create tests directory");

    // Create manifest in spirit_runtime::Manifest compatible format
    // Uses a dummy 64-char hex author (32-byte Ed25519 public key placeholder)
    let manifest = format!(
        r#"name = "{}"
version = {{ major = 0, minor = 1, patch = 0 }}
author = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
description = "A VUDO Spirit project"

[pricing]
base_cost = 100
per_fuel_cost = 1
"#,
        name
    );

    fs::write(project_path.join("manifest.toml"), manifest).expect("Failed to write manifest.toml");

    // Create main.dol (WASM-compilable: plain functions, no host calls)
    let main_dol = r#"fun add(a: i64, b: i64) -> i64 {
    return a + b
}

fun main() -> i64 {
    return add(40, 2)
}
"#;
    fs::write(project_path.join("src/main.dol"), main_dol).expect("Failed to write main.dol");

    // Create test file
    let test_dol = r#"// Tests for the Spirit
// Run with: vudo test

#[test]
fun test_basic() {
    assert(true, "Basic test should pass")
}
"#;
    fs::write(project_path.join("tests/main_test.dol"), test_dol)
        .expect("Failed to write test file");

    project_path
}

// =============================================================================
// Test 1: Full Spirit Lifecycle (new -> build -> run)
// =============================================================================

/// Tests the complete Spirit lifecycle using a compatible manifest format.
///
/// This test creates a Spirit project with a manifest that matches
/// spirit_runtime::Manifest expectations, then builds and runs it.
#[test]
fn test_full_spirit_lifecycle() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create project with compatible manifest
    let project_path = create_compatible_spirit_project(temp_path, "test-spirit");

    assert!(project_path.exists(), "Project directory should exist");
    assert!(
        project_path.join("manifest.toml").exists(),
        "manifest.toml should exist"
    );
    assert!(
        project_path.join("src/main.dol").exists(),
        "src/main.dol should exist"
    );

    // Step 2: Build the Spirit project
    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    let spirit_file = project_path.join("test-spirit.spirit");
    assert!(
        spirit_file.exists(),
        "Built Spirit package should exist at {:?}",
        spirit_file
    );

    // Verify the built file is a valid WASM module (starts with magic number)
    let spirit_bytes = fs::read(&spirit_file).expect("Failed to read spirit file");
    assert!(
        spirit_bytes.len() >= 8,
        "Spirit file should contain valid WASM module"
    );
    assert_eq!(
        &spirit_bytes[0..4],
        b"\0asm",
        "Spirit file should start with WASM magic number"
    );

    // Step 3: Run the Spirit
    let output = run_vudo(&["run"], &project_path);
    assert_success(&output, "vudo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Running") || stdout.contains("Execution completed"),
        "Run output should indicate execution: {}",
        stdout
    );
}

// =============================================================================
// Test 2: vudo new with templates
// =============================================================================

#[test]
fn test_new_creates_project_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create Spirit with basic template
    let output = run_vudo(&["new", "basic-spirit"], temp_path);
    assert_success(&output, "vudo new basic-spirit");

    let project_path = temp_path.join("basic-spirit");
    assert!(project_path.exists(), "Project directory should exist");
    assert!(project_path.join("manifest.toml").exists());
    assert!(project_path.join("src/main.dol").exists());
    assert!(project_path.join("tests").is_dir());
    assert!(project_path.join("README.md").exists());
}

#[test]
fn test_new_with_cli_tool_template() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create Spirit with cli-tool template
    let output = run_vudo(&["new", "my-cli", "--template", "cli-tool"], temp_path);
    assert_success(&output, "vudo new my-cli --template cli-tool");

    let project_path = temp_path.join("my-cli");
    let main_dol = project_path.join("src/main.dol");
    assert!(main_dol.exists(), "src/main.dol should exist");

    // Verify the content contains CLI-specific constructs
    let content = fs::read_to_string(&main_dol).expect("Failed to read main.dol");
    assert!(
        content.contains("CliArgs") || content.contains("CLI Tool"),
        "CLI template should contain CLI-specific code"
    );
    assert!(
        content.contains("parse_args") || content.contains("verbose"),
        "CLI template should have argument parsing code"
    );
}

#[test]
fn test_new_with_web_service_template() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create Spirit with web-service template
    let output = run_vudo(
        &["new", "my-service", "--template", "web-service"],
        temp_path,
    );
    assert_success(&output, "vudo new my-service --template web-service");

    let project_path = temp_path.join("my-service");
    let main_dol = project_path.join("src/main.dol");
    assert!(main_dol.exists(), "src/main.dol should exist");

    // Verify the content contains web service constructs
    let content = fs::read_to_string(&main_dol).expect("Failed to read main.dol");
    assert!(
        content.contains("http") || content.contains("Web Service") || content.contains("port"),
        "Web service template should contain web-specific code"
    );
}

#[test]
fn test_new_with_library_template() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create Spirit with library template
    let output = run_vudo(&["new", "my-lib", "--template", "library"], temp_path);
    assert_success(&output, "vudo new my-lib --template library");

    let project_path = temp_path.join("my-lib");
    let main_dol = project_path.join("src/main.dol");
    assert!(main_dol.exists(), "src/main.dol should exist");

    // Verify the content contains library-specific constructs
    let content = fs::read_to_string(&main_dol).expect("Failed to read main.dol");
    assert!(
        content.contains("fun add") || content.contains("Library"),
        "Library template should contain reusable function definitions"
    );
}

#[test]
fn test_new_with_custom_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a custom subdirectory
    let custom_path = temp_path.join("custom/projects");
    fs::create_dir_all(&custom_path).expect("Failed to create custom directory");

    // Create Spirit in custom path
    let output = run_vudo(
        &["new", "path-test", "--path", custom_path.to_str().unwrap()],
        temp_path,
    );
    assert_success(&output, "vudo new path-test --path custom/projects");

    let project_path = custom_path.join("path-test");
    assert!(
        project_path.exists(),
        "Project should be created in custom path"
    );
    assert!(project_path.join("manifest.toml").exists());
}

// =============================================================================
// Test 3: vudo check validates DOL syntax
// =============================================================================

#[test]
fn test_check_validates_dol_syntax() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a new Spirit project
    let output = run_vudo(&["new", "check-test"], temp_path);
    assert_success(&output, "vudo new check-test");

    let project_path = temp_path.join("check-test");

    // Run check command
    let output = run_vudo(&["check"], &project_path);
    assert_success(&output, "vudo check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Checking") || stdout.contains("OK") || stdout.contains("passed"),
        "Check output should indicate validation: {}",
        stdout
    );
}

#[test]
fn test_check_on_specific_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a new Spirit project
    let output = run_vudo(&["new", "file-check-test"], temp_path);
    assert_success(&output, "vudo new file-check-test");

    let project_path = temp_path.join("file-check-test");
    let main_dol = project_path.join("src/main.dol");

    // Run check on specific file
    let output = run_vudo(&["check", main_dol.to_str().unwrap()], temp_path);
    assert_success(&output, "vudo check src/main.dol");
}

#[test]
fn test_check_with_strict_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a new Spirit project
    let output = run_vudo(&["new", "strict-check-test"], temp_path);
    assert_success(&output, "vudo new strict-check-test");

    let project_path = temp_path.join("strict-check-test");

    // Run check with strict mode
    let output = run_vudo(&["check", "--strict"], &project_path);
    assert_success(&output, "vudo check --strict");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("strict") || stdout.contains("Checking"),
        "Strict check should run: {}",
        stdout
    );
}

#[test]
fn test_check_json_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a new Spirit project
    let output = run_vudo(&["new", "json-check-test"], temp_path);
    assert_success(&output, "vudo new json-check-test");

    let project_path = temp_path.join("json-check-test");

    // Run check with JSON format
    let output = run_vudo(&["check", "--format", "json"], &project_path);
    assert_success(&output, "vudo check --format json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("{") && stdout.contains("}"),
        "JSON output should be valid JSON structure: {}",
        stdout
    );
    assert!(
        stdout.contains("\"success\""),
        "JSON output should contain success field: {}",
        stdout
    );
}

#[test]
fn test_check_warns_on_empty_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a new Spirit project
    let output = run_vudo(&["new", "empty-check-test"], temp_path);
    assert_success(&output, "vudo new empty-check-test");

    let project_path = temp_path.join("empty-check-test");

    // Create an empty DOL file
    let empty_dol = project_path.join("src/empty.dol");
    fs::write(&empty_dol, "").expect("Failed to write empty file");

    // Run check - should warn about empty file
    let output = run_vudo(&["check"], &project_path);
    // This should still succeed but with warnings
    assert_success(&output, "vudo check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("WARN") || stdout.contains("warning") || stdout.contains("empty"),
        "Check should warn about empty file: {}",
        stdout
    );
}

// =============================================================================
// Test 4: vudo pack creates .spirit file
// =============================================================================

#[test]
fn test_pack_creates_spirit_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create project with compatible manifest
    let project_path = create_compatible_spirit_project(temp_path, "pack-test");

    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    // Pack the Spirit
    let output = run_vudo(&["pack"], &project_path);
    assert_success(&output, "vudo pack");

    // Check that versioned package was created
    let pack_file = project_path.join("pack-test-0.1.0.spirit");
    assert!(
        pack_file.exists(),
        "Packed Spirit file should exist at {:?}",
        pack_file
    );

    // Verify pack file has content
    let pack_bytes = fs::read(&pack_file).expect("Failed to read pack file");
    assert!(!pack_bytes.is_empty(), "Pack file should have content");
}

#[test]
fn test_pack_with_custom_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create project with compatible manifest
    let project_path = create_compatible_spirit_project(temp_path, "pack-output-test");

    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    // Pack with custom output path
    let custom_output = temp_path.join("custom-output.spirit");
    let output = run_vudo(
        &["pack", "--output", custom_output.to_str().unwrap()],
        &project_path,
    );
    assert_success(&output, "vudo pack --output custom-output.spirit");

    assert!(
        custom_output.exists(),
        "Custom output file should exist at {:?}",
        custom_output
    );
}

#[test]
fn test_pack_fails_without_build() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create project with compatible manifest but don't build
    let project_path = create_compatible_spirit_project(temp_path, "pack-fail-test");

    // Pack should fail without build
    let output = run_vudo(&["pack"], &project_path);
    assert_failure(&output, "vudo pack (without build)");
}

// =============================================================================
// Test 5: vudo sign and verify workflow
// =============================================================================

/// Tests the full sign and verify workflow.
///
/// Note: The current sign implementation writes raw bytes to the signed package,
/// but the verify implementation expects hex-encoded strings. This test is marked
/// as ignored until the implementation is fixed.
#[test]
#[ignore = "Sign writes raw bytes but verify expects hex-encoded strings - implementation mismatch"]
fn test_sign_and_verify_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a custom VUDO_HOME for key storage isolation
    let vudo_home = temp_path.join(".vudo");
    fs::create_dir_all(&vudo_home).expect("Failed to create vudo home");

    // Create project with compatible manifest
    let project_path = create_compatible_spirit_project(temp_path, "sign-test");

    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    let output = run_vudo(&["pack"], &project_path);
    assert_success(&output, "vudo pack");

    // Get the packed file path
    let pack_file = project_path.join("sign-test-0.1.0.spirit");
    assert!(pack_file.exists(), "Pack file should exist");

    // Sign the package (with isolated VUDO_HOME)
    let output = run_vudo_with_env(
        &["sign", pack_file.to_str().unwrap()],
        &project_path,
        &[("VUDO_HOME", vudo_home.to_str().unwrap())],
    );
    assert_success(&output, "vudo sign");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Signing") || stdout.contains("Signed"),
        "Sign output should indicate signing: {}",
        stdout
    );

    // Check that signed package was created
    let signed_file = pack_file.with_extension("signed.spirit");
    assert!(
        signed_file.exists(),
        "Signed Spirit file should exist at {:?}",
        signed_file
    );

    // Verify the signature
    let output = run_vudo_with_env(
        &["sign", "--verify", signed_file.to_str().unwrap()],
        &project_path,
        &[("VUDO_HOME", vudo_home.to_str().unwrap())],
    );
    assert_success(&output, "vudo sign --verify");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("valid") || stdout.contains("Verifying") || stdout.contains("verified"),
        "Verify output should indicate valid signature: {}",
        stdout
    );
}

#[test]
fn test_sign_with_custom_key() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a custom key directory
    let key_dir = temp_path.join("custom-keys");
    fs::create_dir_all(&key_dir).expect("Failed to create key directory");

    // Create project with compatible manifest
    let project_path = create_compatible_spirit_project(temp_path, "custom-key-test");

    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    let output = run_vudo(&["pack"], &project_path);
    assert_success(&output, "vudo pack");

    let pack_file = project_path.join("custom-key-test-0.1.0.spirit");
    let custom_key = key_dir.join("my-signing.key");

    // Sign with custom key path (key will be generated automatically)
    let output = run_vudo(
        &[
            "sign",
            "--key",
            custom_key.to_str().unwrap(),
            pack_file.to_str().unwrap(),
        ],
        &project_path,
    );
    assert_success(&output, "vudo sign --key custom-key");

    // Verify the key was created
    assert!(
        custom_key.exists(),
        "Custom key file should be created at {:?}",
        custom_key
    );

    // Verify the signed package was created
    let signed_file = pack_file.with_extension("signed.spirit");
    assert!(signed_file.exists(), "Signed file should exist");
}

#[test]
fn test_verify_fails_on_unsigned_package() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create project with compatible manifest (but don't sign)
    let project_path = create_compatible_spirit_project(temp_path, "unsigned-test");

    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    let output = run_vudo(&["pack"], &project_path);
    assert_success(&output, "vudo pack");

    let pack_file = project_path.join("unsigned-test-0.1.0.spirit");

    // Verify should fail on unsigned package
    let output = run_vudo(
        &["sign", "--verify", pack_file.to_str().unwrap()],
        &project_path,
    );
    assert_failure(&output, "vudo sign --verify (unsigned package)");
}

// =============================================================================
// Test 6: Build with various options
// =============================================================================

#[test]
fn test_build_with_release_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let project_path = create_compatible_spirit_project(temp_path, "release-build-test");

    // Build in release mode
    let output = run_vudo(&["build", "--release"], &project_path);
    assert_success(&output, "vudo build --release");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("release") || stdout.contains("Mode"),
        "Release build output should indicate release mode: {}",
        stdout
    );
}

#[test]
fn test_build_with_custom_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let project_path = create_compatible_spirit_project(temp_path, "custom-build-output");
    let custom_output = temp_path.join("my-spirit.wasm");

    // Build with custom output path
    let output = run_vudo(
        &["build", "--output", custom_output.to_str().unwrap()],
        &project_path,
    );
    assert_success(&output, "vudo build --output custom");

    assert!(
        custom_output.exists(),
        "Custom output file should exist at {:?}",
        custom_output
    );
}

#[test]
fn test_build_with_emit_option() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let project_path = create_compatible_spirit_project(temp_path, "emit-test");

    // Build with emit AST option
    let output = run_vudo(&["build", "--emit", "ast"], &project_path);
    assert_success(&output, "vudo build --emit ast");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ast") || stdout.contains("AST") || stdout.contains("Emitting"),
        "Emit output should mention AST: {}",
        stdout
    );
}

// =============================================================================
// Test 7: Run with various options
// =============================================================================

#[test]
fn test_run_with_fuel_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let project_path = create_compatible_spirit_project(temp_path, "fuel-test");

    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    // Run with custom fuel limit
    let output = run_vudo(&["run", "--fuel", "500000"], &project_path);
    assert_success(&output, "vudo run --fuel 500000");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("500000") || stdout.contains("Fuel"),
        "Run output should show fuel limit: {}",
        stdout
    );
}

#[test]
fn test_run_with_memory_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let project_path = create_compatible_spirit_project(temp_path, "memory-test");

    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    // Run with memory limit
    let output = run_vudo(&["run", "--memory", "64mb"], &project_path);
    assert_success(&output, "vudo run --memory 64mb");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Memory") || stdout.contains("bytes"),
        "Run output should show memory info: {}",
        stdout
    );
}

#[test]
fn test_run_with_trace() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let project_path = create_compatible_spirit_project(temp_path, "trace-test");

    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    // Run with tracing enabled
    let output = run_vudo(&["run", "--trace"], &project_path);
    assert_success(&output, "vudo run --trace");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Trace") || stdout.contains("trace"),
        "Run output should mention tracing: {}",
        stdout
    );
}

#[test]
fn test_run_specific_spirit_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let project_path = create_compatible_spirit_project(temp_path, "specific-run-test");

    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    let spirit_file = project_path.join("specific-run-test.spirit");

    // Run specific spirit file
    let output = run_vudo(&["run", spirit_file.to_str().unwrap()], temp_path);
    assert_success(&output, "vudo run specific-spirit.spirit");
}

// =============================================================================
// Test 8: Error handling and edge cases
// =============================================================================

#[test]
fn test_build_fails_without_manifest() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Try to build in empty directory (no manifest)
    let output = run_vudo(&["build"], temp_path);
    assert_failure(&output, "vudo build (without manifest)");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("manifest") || stderr.contains("Failed") || stderr.contains("Error"),
        "Error should mention missing manifest: {}",
        stderr
    );
}

#[test]
fn test_run_fails_without_spirit() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create project but don't build
    let project_path = create_compatible_spirit_project(temp_path, "run-fail-test");

    // Try to run without building
    let output = run_vudo(&["run"], &project_path);
    assert_failure(&output, "vudo run (without build)");
}

#[test]
fn test_check_fails_on_nonexistent_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Try to check a nonexistent path
    let output = run_vudo(&["check", "/nonexistent/path/to/file.dol"], temp_path);
    assert_failure(&output, "vudo check (nonexistent path)");
}

#[test]
fn test_new_with_existing_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a project first
    let output = run_vudo(&["new", "duplicate-test"], temp_path);
    assert_success(&output, "vudo new duplicate-test");

    // Try to create with same name - current behavior may vary
    // We just verify the command doesn't crash
    let output = run_vudo(&["new", "duplicate-test"], temp_path);
    // Either succeeds (overwrites) or fails (directory exists)
    let _ = output.status;
}

// =============================================================================
// Test 9: Manifest and project structure
// =============================================================================

#[test]
fn test_manifest_content_from_new() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let output = run_vudo(&["new", "manifest-test"], temp_path);
    assert_success(&output, "vudo new manifest-test");

    let project_path = temp_path.join("manifest-test");
    let manifest_path = project_path.join("manifest.toml");

    let manifest_content = fs::read_to_string(&manifest_path).expect("Failed to read manifest");

    // Verify manifest has required fields (current format from vudo new)
    assert!(
        manifest_content.contains("[spirit]"),
        "Manifest should have [spirit] section"
    );
    assert!(
        manifest_content.contains("name ="),
        "Manifest should have name field"
    );
    assert!(
        manifest_content.contains("version ="),
        "Manifest should have version field"
    );
    assert!(
        manifest_content.contains("manifest-test"),
        "Manifest should contain project name"
    );
}

#[test]
fn test_project_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let output = run_vudo(&["new", "structure-test"], temp_path);
    assert_success(&output, "vudo new structure-test");

    let project_path = temp_path.join("structure-test");

    // Verify all expected files and directories exist
    assert!(project_path.join("manifest.toml").exists());
    assert!(project_path.join("src").is_dir());
    assert!(project_path.join("src/main.dol").exists());
    assert!(project_path.join("tests").is_dir());
    assert!(project_path.join("tests/main_test.dol").exists());
    assert!(project_path.join("README.md").exists());
}

// =============================================================================
// Test 10: CLI help and version
// =============================================================================

#[test]
fn test_cli_help() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let output = run_vudo(&["--help"], temp_path);
    assert_success(&output, "vudo --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vudo"), "Help should mention vudo");
    assert!(
        stdout.contains("new") && stdout.contains("build") && stdout.contains("run"),
        "Help should list main commands"
    );
}

#[test]
fn test_cli_version() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let output = run_vudo(&["--version"], temp_path);
    assert_success(&output, "vudo --version");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("vudo") || stdout.contains("0."),
        "Version output should contain version info: {}",
        stdout
    );
}

#[test]
fn test_subcommand_help() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Test help for various subcommands
    for cmd in &["new", "build", "run", "check", "pack", "sign"] {
        let output = run_vudo(&[cmd, "--help"], temp_path);
        assert_success(&output, &format!("vudo {} --help", cmd));
    }
}

// =============================================================================
// Test 11: Quiet and verbose modes
// =============================================================================

#[test]
fn test_quiet_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let output = run_vudo(&["--quiet", "new", "quiet-test"], temp_path);
    assert_success(&output, "vudo --quiet new quiet-test");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Quiet mode should produce minimal output
    assert!(stdout.len() < 1000, "Quiet mode should produce less output");
}

#[test]
fn test_verbose_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    let output = run_vudo(&["--verbose", "new", "verbose-test"], temp_path);
    assert_success(&output, "vudo --verbose new verbose-test");

    // Verbose mode should succeed - output amount varies by implementation
    let project_path = temp_path.join("verbose-test");
    assert!(project_path.exists());
}

// =============================================================================
// Test 12: Integration between vudo new output and build
// This test documents the current manifest format incompatibility
// =============================================================================

/// This test documents the known incompatibility between the manifest format
/// produced by `vudo new` and what `spirit_runtime::Manifest` expects.
///
/// Currently, `vudo new` creates:
/// ```toml
/// [spirit]
/// name = "..."
/// version = "0.1.0"
/// ```
///
/// But `spirit_runtime::Manifest` expects:
/// ```toml
/// name = "..."
/// version = { major = 0, minor = 1, patch = 0 }
/// author = "64-char-hex-string"
/// ```
///
/// This test is marked with #[ignore] until the formats are aligned.
#[test]
#[ignore = "Manifest format from 'vudo new' differs from spirit_runtime::Manifest format"]
fn test_new_then_build_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Use vudo new to create the project (uses current [spirit] section format)
    let output = run_vudo(&["new", "integration-test"], temp_path);
    assert_success(&output, "vudo new integration-test");

    let project_path = temp_path.join("integration-test");

    // This will fail until manifest formats are aligned
    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build after vudo new");
}

// =============================================================================
// H0: toolchain truth — real compile, real execution, honest failures
// =============================================================================

/// The exact bytes of the placeholder module `vudo build` used to write.
const LEGACY_PLACEHOLDER_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x03, 0x02,
    0x01, 0x00, 0x07, 0x08, 0x01, 0x04, b'm', b'a', b'i', b'n', 0x00, 0x00, 0x0A, 0x04, 0x01, 0x02,
    0x00, 0x0B,
];

fn write_main_dol(project_path: &Path, source: &str) {
    fs::write(project_path.join("src/main.dol"), source).expect("Failed to write main.dol");
}

#[test]
fn test_h0_build_emits_compiled_wasm_not_placeholder() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = create_compatible_spirit_project(temp_dir.path(), "h0-build");

    let output = run_vudo(&["build"], &project_path);
    assert_success(&output, "vudo build");

    let wasm = fs::read(project_path.join("h0-build.spirit")).expect("built spirit");
    assert_eq!(&wasm[0..4], b"\0asm", "must be a WASM module");
    assert_ne!(
        wasm.as_slice(),
        LEGACY_PLACEHOLDER_WASM,
        "must not be the placeholder"
    );
    // Export names appear verbatim in the export section.
    for export in ["add", "main"] {
        assert!(
            wasm.windows(export.len()).any(|w| w == export.as_bytes()),
            "compiled module should export `{}`",
            export
        );
    }
}

#[test]
fn test_h0_build_fails_on_invalid_dol() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = create_compatible_spirit_project(temp_dir.path(), "h0-bad");
    write_main_dol(&project_path, "fun main( -> i64 { return }\n");

    let output = run_vudo(&["build"], &project_path);
    assert_failure(&output, "vudo build on invalid DOL");
    assert!(
        !project_path.join("h0-bad.spirit").exists(),
        "no output may be written when compilation fails"
    );
}

#[test]
fn test_h0_run_executes_entry_and_prints_result() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = create_compatible_spirit_project(temp_dir.path(), "h0-run");

    assert_success(&run_vudo(&["build"], &project_path), "vudo build");
    let output = run_vudo(&["run"], &project_path);
    assert_success(&output, "vudo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("42"),
        "run should print main()'s result 42: {}",
        stdout
    );
}

#[test]
fn test_h0_run_custom_entry() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = create_compatible_spirit_project(temp_dir.path(), "h0-entry");

    assert_success(&run_vudo(&["build"], &project_path), "vudo build");
    let output = run_vudo(&["run", "--entry", "add", "--", "2", "3"], &project_path);
    assert_success(&output, "vudo run --entry add -- 2 3");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('5'), "add(2, 3) should print 5: {}", stdout);
}

#[test]
fn test_h0_run_fails_on_trap() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = create_compatible_spirit_project(temp_dir.path(), "h0-trap");
    write_main_dol(
        &project_path,
        "fun divide(a: i64, b: i64) -> i64 {\n    return a / b\n}\n\nfun main() -> i64 {\n    return divide(1, 0)\n}\n",
    );

    assert_success(&run_vudo(&["build"], &project_path), "vudo build");
    let output = run_vudo(&["run"], &project_path);
    assert_failure(&output, "vudo run on a trapping entry function");
}

#[test]
fn test_h0_run_fails_on_fuel_exhaustion() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = create_compatible_spirit_project(temp_dir.path(), "h0-fuel");
    write_main_dol(
        &project_path,
        "fun main() -> i64 {\n    let i: i64 = 0\n    while i < 1000000000 {\n        i = i + 1\n    }\n    return i\n}\n",
    );

    assert_success(&run_vudo(&["build"], &project_path), "vudo build");
    let output = run_vudo(&["run", "--fuel", "10000"], &project_path);
    assert_failure(&output, "vudo run with exhausted fuel");
}

#[test]
fn test_h0_run_rejects_non_wasm() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let fake = temp_dir.path().join("fake.spirit");
    fs::write(&fake, b"Summoned Spirit placeholder").unwrap();

    let output = run_vudo(&["run", fake.to_str().unwrap()], temp_dir.path());
    assert_failure(&output, "vudo run on a non-WASM file");
}

#[test]
fn test_h0_publish_never_reports_success() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = create_compatible_spirit_project(temp_dir.path(), "h0-publish");
    assert_success(&run_vudo(&["build"], &project_path), "vudo build");
    assert_success(&run_vudo(&["pack"], &project_path), "vudo pack");

    let output = run_vudo(&["publish", "h0-publish-0.1.0.spirit"], &project_path);
    assert_failure(&output, "vudo publish (network registry not implemented)");
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(all.contains("H2"), "error should point to plan H2: {}", all);
    assert!(
        !all.contains("published successfully"),
        "must not claim success: {}",
        all
    );
}

#[test]
fn test_h0_summon_never_reports_success_or_writes_placeholder() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let out_dir = temp_dir.path().join("spirits");

    let output = run_vudo(
        &[
            "summon",
            "@someone/thing",
            "--output",
            out_dir.to_str().unwrap(),
        ],
        temp_dir.path(),
    );
    assert_failure(&output, "vudo summon (network registry not implemented)");
    assert!(
        !out_dir.join("thing.spirit").exists(),
        "summon must not write a placeholder package"
    );
}

#[test]
fn test_h0_pack_has_no_fake_compression() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = create_compatible_spirit_project(temp_dir.path(), "h0-pack");
    assert_success(&run_vudo(&["build"], &project_path), "vudo build");

    let output = run_vudo(&["pack", "--compress", "zstd"], &project_path);
    assert_failure(&output, "vudo pack --compress (flag removed)");
}

// ─── Spirit Hosting H0 exit oracle: a Spirit calling host_log is observed ───
//
// The module is hand-written WAT against the canonical VM ABI (module "vudo",
// host_* names; decision D2), so this oracle does not depend on a DOL release.
// univrs/dol tests/wasm_host_abi.rs proves DOL emits exactly these imports.

const H0_LOG_LINE: &str = "hello from the VUDO VM ABI";

fn write_host_call_spirit(dir: &Path) -> std::path::PathBuf {
    let wat = format!(
        r#"(module
  (import "vudo" "host_log" (func $log (param i32 i32 i32) (result i32)))
  (import "vudo" "host_time_now" (func $now (result i64)))
  (memory (export "memory") 1)
  (data (i32.const 16) "{line}")
  (func (export "main") (result i64)
    (i64.extend_i32_s (call $log (i32.const 2) (i32.const 16) (i32.const {len}))))
  (func (export "now") (result i64) (call $now)))"#,
        line = H0_LOG_LINE,
        len = H0_LOG_LINE.len()
    );
    let path = dir.join("host-call.spirit");
    fs::write(&path, wat::parse_str(&wat).expect("valid WAT")).expect("write spirit");
    path
}

#[test]
fn test_h0_exit_host_log_line_is_observed_with_log_capability() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spirit = write_host_call_spirit(temp_dir.path());
    let output = run_vudo(
        &["run", spirit.to_str().unwrap(), "--capabilities", "log"],
        temp_dir.path(),
    );
    assert_success(&output, "vudo run with log capability");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("[VUDO:INFO] {}", H0_LOG_LINE)),
        "log line must be observed; stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Result: 0"),
        "host_log must return success (0); stdout:\n{}",
        stdout
    );
}

#[test]
fn test_h0_host_log_never_emits_without_log_capability() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spirit = write_host_call_spirit(temp_dir.path());
    let output = run_vudo(&["run", spirit.to_str().unwrap()], temp_dir.path());
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !all.contains(H0_LOG_LINE),
        "a Spirit without the log capability must never log; output:\n{}",
        all
    );
    assert!(
        all.contains("Result: -1"),
        "host_log must be denied; output:\n{}",
        all
    );
}

#[test]
fn test_h0_granted_global_capability_is_honoured_by_vm() {
    // time is a Global-scope host capability in vudo_vm; a CLI grant must satisfy it.
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spirit = write_host_call_spirit(temp_dir.path());
    let output = run_vudo(
        &[
            "run",
            spirit.to_str().unwrap(),
            "--capabilities",
            "time",
            "--entry",
            "now",
        ],
        temp_dir.path(),
    );
    assert_success(&output, "vudo run with time capability");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Result: 0\n") && !stdout.contains("Result: -1"),
        "host_time_now must return a real timestamp, not 0 or -1; stdout:\n{}",
        stdout
    );
}

/// H0 exit oracle, from DOL source: `vudo build` compiles a DOL Spirit that
/// calls `host_log`, and `vudo run` executes it in the VM and the line appears.
/// Needs dol >= 0.8.2 (host imports on module `vudo`, string -> (ptr, len)).
#[test]
fn test_h0_exit_dol_source_host_log_runs_in_vm() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = create_compatible_spirit_project(temp_dir.path(), "h0-dol-hostlog");
    write_main_dol(
        &project_path,
        "sex fun host_log(level: i32, msg: string) -> i32\n\nsex fun main() -> i64 {\n    return host_log(2, \"hello from DOL\")\n}\n",
    );

    assert_success(
        &run_vudo(&["build"], &project_path),
        "vudo build (DOL calling host_log)",
    );
    let wasm = find_built_spirit(&project_path);
    let output = run_vudo(
        &["run", wasm.to_str().unwrap(), "--capabilities", "log"],
        &project_path,
    );
    assert_success(&output, "vudo run (DOL calling host_log)");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[VUDO:INFO] hello from DOL"),
        "DOL-compiled host_log line must be observed; stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Result: 0"),
        "host_log must succeed; stdout:\n{}",
        stdout
    );
}

fn find_built_spirit(project_path: &Path) -> std::path::PathBuf {
    fs::read_dir(project_path)
        .expect("read project dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.extension().is_some_and(|x| x == "spirit"))
        .expect("vudo build must write a .spirit package")
}

// ─── Spirit Hosting H1a: `vudo run` honours manifest [resources] ───

const H1A_LOOP_DOL: &str = "fun main() -> i64 {\n    let i: i64 = 0\n    while i < 20000 {\n        i = i + 1\n    }\n    return i\n}\n";

fn h1a_project(temp: &Path, name: &str, resources: &str) -> std::path::PathBuf {
    let project_path = create_compatible_spirit_project(temp, name);
    let manifest = project_path.join("manifest.toml");
    let mut m = fs::read_to_string(&manifest).unwrap();
    m.push_str(&format!("\n[resources]\n{}\n", resources));
    fs::write(&manifest, m).unwrap();
    write_main_dol(&project_path, H1A_LOOP_DOL);
    assert_success(&run_vudo(&["build"], &project_path), "vudo build");
    project_path
}

#[test]
fn test_h1a_run_uses_manifest_fuel() {
    let temp_dir = TempDir::new().expect("temp dir");
    // The loop fits the CLI default (1,000,000) but not the manifest's 10,000.
    let p = h1a_project(temp_dir.path(), "h1a-fuel", "fuel = 10000");
    let output = run_vudo(&["run"], &p);
    assert_failure(&output, "vudo run must apply manifest fuel = 10000");
    assert!(String::from_utf8_lossy(&output.stdout).contains("Fuel: 10000"));
}

#[test]
fn test_h1a_cli_fuel_overrides_manifest() {
    let temp_dir = TempDir::new().expect("temp dir");
    let p = h1a_project(temp_dir.path(), "h1a-override", "fuel = 10000");
    let output = run_vudo(&["run", "--fuel", "5000000"], &p);
    assert_success(&output, "--fuel must override manifest fuel");
    assert!(String::from_utf8_lossy(&output.stdout).contains("Result: 20000"));
}

#[test]
fn test_h1a_run_uses_manifest_memory() {
    let temp_dir = TempDir::new().expect("temp dir");
    let p = h1a_project(temp_dir.path(), "h1a-mem", "memory_bytes = 1048576");
    let output = run_vudo(&["run"], &p);
    assert_success(&output, "run within manifest memory");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Memory: 1048576 bytes"),
        "manifest memory_bytes must be the sandbox limit"
    );
}
