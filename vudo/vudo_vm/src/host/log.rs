//! Host Logging Functions
//!
//! Provides logging capabilities for WASM sandboxes.

use super::{CapabilityScope, CapabilitySet, CapabilityType, HostCallResult};
use std::fmt;

/// Log level for host logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl LogLevel {
    /// Create LogLevel from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(LogLevel::Trace),
            1 => Some(LogLevel::Debug),
            2 => Some(LogLevel::Info),
            3 => Some(LogLevel::Warn),
            4 => Some(LogLevel::Error),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Maximum log message length
const MAX_LOG_MESSAGE_LENGTH: usize = 64 * 1024; // 64KB

/// Log a message from a WASM sandbox
///
/// Requires ActuatorLog capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `level` - Log level (Trace, Debug, Info, Warn, Error)
/// * `message` - Message to log
///
/// # Returns
/// HostCallResult indicating success or error
pub fn host_log(caps: &CapabilitySet, level: LogLevel, message: &str) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::ActuatorLog, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::ActuatorLog);
    }

    // Validate message length
    if message.len() > MAX_LOG_MESSAGE_LENGTH {
        return HostCallResult::error(format!(
            "Log message exceeds maximum length of {} bytes",
            MAX_LOG_MESSAGE_LENGTH
        ));
    }

    // Truncate message if it's too long for display (but this shouldn't happen due to above check)
    let truncated = if message.len() > MAX_LOG_MESSAGE_LENGTH {
        &message[..MAX_LOG_MESSAGE_LENGTH]
    } else {
        message
    };

    // Log the message using the appropriate level
    // In a real implementation, this would integrate with a proper logging framework
    match level {
        LogLevel::Trace => eprintln!("[VUDO:TRACE] {}", truncated),
        LogLevel::Debug => eprintln!("[VUDO:DEBUG] {}", truncated),
        LogLevel::Info => println!("[VUDO:INFO] {}", truncated),
        LogLevel::Warn => println!("[VUDO:WARN] {}", truncated),
        LogLevel::Error => eprintln!("[VUDO:ERROR] {}", truncated),
    }

    HostCallResult::success()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityGrant, MINIMAL_CAPABILITIES};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_capset() -> CapabilitySet {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut cap_set = CapabilitySet::new();
        for &cap_type in MINIMAL_CAPABILITIES {
            let grant = CapabilityGrant::new(
                1,
                cap_type,
                CapabilityScope::Global,
                [0u8; 32],
                [1u8; 32],
                now,
                None,
                [0u8; 64],
            );
            cap_set.add_grant(grant);
        }
        cap_set
    }

    fn create_unrestricted_capset() -> CapabilitySet {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut cap_set = CapabilitySet::new();
        let grant = CapabilityGrant::new(
            1,
            CapabilityType::Unrestricted,
            CapabilityScope::Global,
            [0u8; 32],
            [1u8; 32],
            now,
            None,
            [0u8; 64],
        );
        cap_set.add_grant(grant);
        cap_set
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_log_level_from_u8() {
        assert_eq!(LogLevel::from_u8(0), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_u8(1), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_u8(2), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_u8(3), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_u8(4), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_u8(5), None);
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Trace.as_str(), "TRACE");
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_host_log_with_capability() {
        let caps = create_test_capset();
        let result = host_log(&caps, LogLevel::Info, "test message");

        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_host_log_without_capability() {
        let caps = CapabilitySet::new();
        let result = host_log(&caps, LogLevel::Info, "test message");

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_log_all_levels() {
        let caps = create_test_capset();

        for level in [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            let result = host_log(&caps, level, &format!("Test {} message", level));
            assert!(result.success);
        }
    }

    #[test]
    fn test_host_log_message_too_long() {
        let caps = create_test_capset();
        let long_message = "x".repeat(MAX_LOG_MESSAGE_LENGTH + 1);
        let result = host_log(&caps, LogLevel::Info, &long_message);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("exceeds maximum length"));
    }

    #[test]
    fn test_host_log_empty_message() {
        let caps = create_test_capset();
        let result = host_log(&caps, LogLevel::Info, "");

        assert!(result.success);
    }

    #[test]
    fn test_host_log_with_unrestricted() {
        let caps = create_unrestricted_capset();
        let result = host_log(&caps, LogLevel::Info, "test message");

        assert!(result.success);
    }
}
