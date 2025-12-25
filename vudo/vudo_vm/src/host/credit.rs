//! Host Credit Functions
//!
//! Provides credit management capabilities for WASM sandboxes.
//! Credits are used for resource consumption metering in the VUDO system.
//! Credits are tied to Ed25519 public keys (32 bytes).

use super::{CapabilityScope, CapabilitySet, CapabilityType, HostCallResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Size of an Ed25519 public key in bytes
pub const PUBLIC_KEY_SIZE: usize = 32;

/// Maximum amount of credits that can be transferred in a single operation
pub const MAX_TRANSFER_AMOUNT: u64 = 1_000_000_000_000; // 1 trillion

/// Maximum amount of credits that can be reserved in a single operation
pub const MAX_RESERVE_AMOUNT: u64 = 100_000_000_000; // 100 billion

/// Ed25519 public key type alias
pub type PublicKey = [u8; PUBLIC_KEY_SIZE];

// ═══════════════════════════════════════════════════════════════════════════
// CREDIT BACKEND TRAIT
// ═══════════════════════════════════════════════════════════════════════════

/// Credit backend trait
///
/// Implementations provide the actual credit accounting mechanism
/// (in-memory, database, blockchain, etc.)
pub trait CreditBackend: Send + Sync {
    /// Get the credit balance for a public key
    ///
    /// Returns:
    /// - Ok(balance) - The current balance
    /// - Err(msg) - On ledger error
    fn balance(&self, account: &PublicKey) -> Result<u64, String>;

    /// Transfer credits from one account to another
    ///
    /// Returns:
    /// - Ok(()) - On successful transfer
    /// - Err(msg) - On insufficient funds or ledger error
    fn transfer(&self, from: &PublicKey, to: &PublicKey, amount: u64) -> Result<(), String>;

    /// Reserve credits for a pending operation
    ///
    /// Reserved credits are deducted from the balance but held in escrow.
    /// They can be released back to the account or consumed.
    ///
    /// Returns:
    /// - Ok(reservation_id) - A unique ID for this reservation
    /// - Err(msg) - On insufficient funds or ledger error
    fn reserve(&self, account: &PublicKey, amount: u64) -> Result<u64, String>;

    /// Release a previously made reservation back to the account
    ///
    /// Returns:
    /// - Ok(()) - On successful release
    /// - Err(msg) - If reservation not found or ledger error
    fn release_reservation(&self, reservation_id: u64) -> Result<(), String>;

    /// Consume a reservation (credits are permanently deducted)
    ///
    /// Returns:
    /// - Ok(()) - On successful consumption
    /// - Err(msg) - If reservation not found or ledger error
    fn consume_reservation(&self, reservation_id: u64) -> Result<(), String>;

    /// Get the total reserved credits for an account
    ///
    /// Returns:
    /// - Ok(reserved_amount) - The total reserved credits
    /// - Err(msg) - On ledger error
    fn reserved_balance(&self, account: &PublicKey) -> Result<u64, String>;

    /// Get the available balance (balance minus reserved)
    ///
    /// Returns:
    /// - Ok(available) - The available balance
    /// - Err(msg) - On ledger error
    fn available_balance(&self, account: &PublicKey) -> Result<u64, String>;

    /// Credit an account (for testing and initial funding)
    ///
    /// Returns:
    /// - Ok(()) - On successful credit
    /// - Err(msg) - On ledger error
    fn credit(&self, account: &PublicKey, amount: u64) -> Result<(), String>;
}

// ═══════════════════════════════════════════════════════════════════════════
// IN-MEMORY CREDIT LEDGER
// ═══════════════════════════════════════════════════════════════════════════

/// Reservation entry in the ledger
#[derive(Debug, Clone)]
struct Reservation {
    account: PublicKey,
    amount: u64,
    active: bool,
}

/// In-memory credit ledger implementation
///
/// This is a simple HashMap-based ledger for testing and development.
/// For production, use a persistent or distributed ledger backend.
#[derive(Debug)]
pub struct InMemoryCreditLedger {
    /// Account balances: public_key -> balance
    balances: Arc<RwLock<HashMap<PublicKey, u64>>>,
    /// Active reservations: reservation_id -> Reservation
    reservations: Arc<RwLock<HashMap<u64, Reservation>>>,
    /// Next reservation ID
    next_reservation_id: Arc<RwLock<u64>>,
}

impl InMemoryCreditLedger {
    /// Create a new in-memory credit ledger
    pub fn new() -> Self {
        Self {
            balances: Arc::new(RwLock::new(HashMap::new())),
            reservations: Arc::new(RwLock::new(HashMap::new())),
            next_reservation_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Create a ledger with initial balances
    pub fn with_balances(initial: Vec<(PublicKey, u64)>) -> Self {
        let ledger = Self::new();
        {
            let mut balances = ledger.balances.write().unwrap();
            for (account, amount) in initial {
                balances.insert(account, amount);
            }
        }
        ledger
    }
}

impl Default for InMemoryCreditLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for InMemoryCreditLedger {
    fn clone(&self) -> Self {
        Self {
            balances: Arc::clone(&self.balances),
            reservations: Arc::clone(&self.reservations),
            next_reservation_id: Arc::clone(&self.next_reservation_id),
        }
    }
}

impl CreditBackend for InMemoryCreditLedger {
    fn balance(&self, account: &PublicKey) -> Result<u64, String> {
        let balances = self
            .balances
            .read()
            .map_err(|e| format!("Lock error: {}", e))?;
        Ok(*balances.get(account).unwrap_or(&0))
    }

    fn transfer(&self, from: &PublicKey, to: &PublicKey, amount: u64) -> Result<(), String> {
        if amount == 0 {
            return Err("Transfer amount must be greater than zero".to_string());
        }

        let mut balances = self
            .balances
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        // Get available balance (total - reserved)
        let from_balance = *balances.get(from).unwrap_or(&0);
        let reserved = self.get_reserved_for_account(from)?;
        let available = from_balance.saturating_sub(reserved);

        if available < amount {
            return Err(format!(
                "Insufficient available credits: have {}, need {}, reserved {}",
                available, amount, reserved
            ));
        }

        // Deduct from sender
        *balances.entry(*from).or_insert(0) -= amount;

        // Add to receiver
        *balances.entry(*to).or_insert(0) = balances
            .get(to)
            .unwrap_or(&0)
            .checked_add(amount)
            .ok_or("Credit overflow")?;

        Ok(())
    }

    fn reserve(&self, account: &PublicKey, amount: u64) -> Result<u64, String> {
        if amount == 0 {
            return Err("Reserve amount must be greater than zero".to_string());
        }

        // Check available balance
        let available = self.available_balance(account)?;
        if available < amount {
            return Err(format!(
                "Insufficient available credits for reservation: have {}, need {}",
                available, amount
            ));
        }

        // Generate reservation ID
        let reservation_id = {
            let mut next_id = self
                .next_reservation_id
                .write()
                .map_err(|e| format!("Lock error: {}", e))?;
            let id = *next_id;
            *next_id = next_id.checked_add(1).ok_or("Reservation ID overflow")?;
            id
        };

        // Create reservation
        let mut reservations = self
            .reservations
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;
        reservations.insert(
            reservation_id,
            Reservation {
                account: *account,
                amount,
                active: true,
            },
        );

        Ok(reservation_id)
    }

    fn release_reservation(&self, reservation_id: u64) -> Result<(), String> {
        let mut reservations = self
            .reservations
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        match reservations.get_mut(&reservation_id) {
            Some(reservation) if reservation.active => {
                reservation.active = false;
                Ok(())
            }
            Some(_) => Err(format!(
                "Reservation {} already released or consumed",
                reservation_id
            )),
            None => Err(format!("Reservation {} not found", reservation_id)),
        }
    }

    fn consume_reservation(&self, reservation_id: u64) -> Result<(), String> {
        let reservation = {
            let mut reservations = self
                .reservations
                .write()
                .map_err(|e| format!("Lock error: {}", e))?;

            match reservations.get_mut(&reservation_id) {
                Some(r) if r.active => {
                    r.active = false;
                    Some((r.account, r.amount))
                }
                Some(_) => {
                    return Err(format!(
                        "Reservation {} already released or consumed",
                        reservation_id
                    ))
                }
                None => return Err(format!("Reservation {} not found", reservation_id)),
            }
        };

        if let Some((account, amount)) = reservation {
            let mut balances = self
                .balances
                .write()
                .map_err(|e| format!("Lock error: {}", e))?;
            *balances.entry(account).or_insert(0) =
                balances.get(&account).unwrap_or(&0).saturating_sub(amount);
        }

        Ok(())
    }

    fn reserved_balance(&self, account: &PublicKey) -> Result<u64, String> {
        self.get_reserved_for_account(account)
    }

    fn available_balance(&self, account: &PublicKey) -> Result<u64, String> {
        let total = self.balance(account)?;
        let reserved = self.reserved_balance(account)?;
        Ok(total.saturating_sub(reserved))
    }

    fn credit(&self, account: &PublicKey, amount: u64) -> Result<(), String> {
        let mut balances = self
            .balances
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;
        *balances.entry(*account).or_insert(0) = balances
            .get(account)
            .unwrap_or(&0)
            .checked_add(amount)
            .ok_or("Credit overflow")?;
        Ok(())
    }
}

impl InMemoryCreditLedger {
    /// Helper to get reserved credits for an account
    fn get_reserved_for_account(&self, account: &PublicKey) -> Result<u64, String> {
        let reservations = self
            .reservations
            .read()
            .map_err(|e| format!("Lock error: {}", e))?;
        let reserved: u64 = reservations
            .values()
            .filter(|r| r.active && &r.account == account)
            .map(|r| r.amount)
            .sum();
        Ok(reserved)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOST CREDIT FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Get current credit balance for the calling account
///
/// Requires ActuatorCredit capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `ledger` - Credit ledger backend
/// * `account` - The account (Ed25519 public key) to check balance for
///
/// # Returns
/// HostCallResult with balance as bytes (u64 in little-endian) or error
pub fn host_credit_balance(
    caps: &CapabilitySet,
    ledger: &dyn CreditBackend,
    account: &PublicKey,
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::ActuatorCredit, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::ActuatorCredit);
    }

    // Get balance
    match ledger.balance(account) {
        Ok(balance) => {
            let bytes = balance.to_le_bytes().to_vec();
            HostCallResult::success_with_value(bytes)
        }
        Err(e) => HostCallResult::error(format!("Credit balance error: {}", e)),
    }
}

/// Transfer credits from one account to another
///
/// Requires ActuatorCredit capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `ledger` - Credit ledger backend
/// * `from` - Source account (Ed25519 public key)
/// * `to` - Destination account (Ed25519 public key)
/// * `amount` - Amount of credits to transfer
///
/// # Returns
/// HostCallResult indicating success or error
pub fn host_credit_transfer(
    caps: &CapabilitySet,
    ledger: &dyn CreditBackend,
    from: &PublicKey,
    to: &PublicKey,
    amount: u64,
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::ActuatorCredit, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::ActuatorCredit);
    }

    // Validate amount
    if amount == 0 {
        return HostCallResult::error("Transfer amount must be greater than zero");
    }

    if amount > MAX_TRANSFER_AMOUNT {
        return HostCallResult::error(format!(
            "Transfer amount exceeds maximum of {} credits",
            MAX_TRANSFER_AMOUNT
        ));
    }

    // Validate accounts are different
    if from == to {
        return HostCallResult::error("Cannot transfer to the same account");
    }

    // Perform transfer
    match ledger.transfer(from, to, amount) {
        Ok(()) => HostCallResult::success(),
        Err(e) => HostCallResult::error(format!("Credit transfer error: {}", e)),
    }
}

/// Reserve credits for a pending operation
///
/// Reserved credits are held in escrow and cannot be spent until
/// the reservation is released or consumed.
///
/// Requires ActuatorCredit capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `ledger` - Credit ledger backend
/// * `account` - Account to reserve credits from (Ed25519 public key)
/// * `amount` - Amount of credits to reserve
///
/// # Returns
/// HostCallResult with reservation_id as bytes (u64 in little-endian) or error
pub fn host_credit_reserve(
    caps: &CapabilitySet,
    ledger: &dyn CreditBackend,
    account: &PublicKey,
    amount: u64,
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::ActuatorCredit, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::ActuatorCredit);
    }

    // Validate amount
    if amount == 0 {
        return HostCallResult::error("Reserve amount must be greater than zero");
    }

    if amount > MAX_RESERVE_AMOUNT {
        return HostCallResult::error(format!(
            "Reserve amount exceeds maximum of {} credits",
            MAX_RESERVE_AMOUNT
        ));
    }

    // Perform reservation
    match ledger.reserve(account, amount) {
        Ok(reservation_id) => {
            let bytes = reservation_id.to_le_bytes().to_vec();
            HostCallResult::success_with_value(bytes)
        }
        Err(e) => HostCallResult::error(format!("Credit reserve error: {}", e)),
    }
}

/// Release a previously made credit reservation
///
/// Released credits return to the available balance.
///
/// Requires ActuatorCredit capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `ledger` - Credit ledger backend
/// * `reservation_id` - The reservation ID to release
///
/// # Returns
/// HostCallResult indicating success or error
pub fn host_credit_release(
    caps: &CapabilitySet,
    ledger: &dyn CreditBackend,
    reservation_id: u64,
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::ActuatorCredit, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::ActuatorCredit);
    }

    // Release reservation
    match ledger.release_reservation(reservation_id) {
        Ok(()) => HostCallResult::success(),
        Err(e) => HostCallResult::error(format!("Credit release error: {}", e)),
    }
}

/// Consume a credit reservation (permanently deduct reserved credits)
///
/// Consumed credits are permanently removed from the account.
///
/// Requires ActuatorCredit capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `ledger` - Credit ledger backend
/// * `reservation_id` - The reservation ID to consume
///
/// # Returns
/// HostCallResult indicating success or error
pub fn host_credit_consume(
    caps: &CapabilitySet,
    ledger: &dyn CreditBackend,
    reservation_id: u64,
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::ActuatorCredit, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::ActuatorCredit);
    }

    // Consume reservation
    match ledger.consume_reservation(reservation_id) {
        Ok(()) => HostCallResult::success(),
        Err(e) => HostCallResult::error(format!("Credit consume error: {}", e)),
    }
}

/// Get available credit balance (total - reserved)
///
/// Requires ActuatorCredit capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `ledger` - Credit ledger backend
/// * `account` - The account (Ed25519 public key) to check
///
/// # Returns
/// HostCallResult with available balance as bytes (u64 in little-endian) or error
pub fn host_credit_available(
    caps: &CapabilitySet,
    ledger: &dyn CreditBackend,
    account: &PublicKey,
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::ActuatorCredit, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::ActuatorCredit);
    }

    // Get available balance
    match ledger.available_balance(account) {
        Ok(available) => {
            let bytes = available.to_le_bytes().to_vec();
            HostCallResult::success_with_value(bytes)
        }
        Err(e) => HostCallResult::error(format!("Credit available error: {}", e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityGrant;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Test accounts (simulated Ed25519 public keys)
    fn alice_key() -> PublicKey {
        let mut key = [0u8; 32];
        key[0] = 0xAA;
        key
    }

    fn bob_key() -> PublicKey {
        let mut key = [0u8; 32];
        key[0] = 0xBB;
        key
    }

    fn charlie_key() -> PublicKey {
        let mut key = [0u8; 32];
        key[0] = 0xCC;
        key
    }

    fn create_credit_caps() -> CapabilitySet {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut cap_set = CapabilitySet::new();
        let grant = CapabilityGrant::new(
            1,
            CapabilityType::ActuatorCredit,
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

    // ═══════════════════════════════════════════════════════════════════════
    // IN-MEMORY LEDGER TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_ledger_initial_balance() {
        let ledger = InMemoryCreditLedger::new();
        let balance = ledger.balance(&alice_key()).unwrap();
        assert_eq!(balance, 0);
    }

    #[test]
    fn test_ledger_credit_account() {
        let ledger = InMemoryCreditLedger::new();
        ledger.credit(&alice_key(), 1000).unwrap();

        let balance = ledger.balance(&alice_key()).unwrap();
        assert_eq!(balance, 1000);
    }

    #[test]
    fn test_ledger_with_initial_balances() {
        let ledger =
            InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000), (bob_key(), 500)]);

        assert_eq!(ledger.balance(&alice_key()).unwrap(), 1000);
        assert_eq!(ledger.balance(&bob_key()).unwrap(), 500);
        assert_eq!(ledger.balance(&charlie_key()).unwrap(), 0);
    }

    #[test]
    fn test_ledger_transfer_success() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        ledger.transfer(&alice_key(), &bob_key(), 300).unwrap();

        assert_eq!(ledger.balance(&alice_key()).unwrap(), 700);
        assert_eq!(ledger.balance(&bob_key()).unwrap(), 300);
    }

    #[test]
    fn test_ledger_transfer_insufficient_funds() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 100)]);

        let result = ledger.transfer(&alice_key(), &bob_key(), 200);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient"));
    }

    #[test]
    fn test_ledger_transfer_zero_amount() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let result = ledger.transfer(&alice_key(), &bob_key(), 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater than zero"));
    }

    #[test]
    fn test_ledger_reserve_success() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let reservation_id = ledger.reserve(&alice_key(), 300).unwrap();
        assert!(reservation_id > 0);

        // Balance unchanged, but available reduced
        assert_eq!(ledger.balance(&alice_key()).unwrap(), 1000);
        assert_eq!(ledger.reserved_balance(&alice_key()).unwrap(), 300);
        assert_eq!(ledger.available_balance(&alice_key()).unwrap(), 700);
    }

    #[test]
    fn test_ledger_reserve_insufficient_available() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        // First reservation succeeds
        ledger.reserve(&alice_key(), 800).unwrap();

        // Second reservation fails (only 200 available)
        let result = ledger.reserve(&alice_key(), 300);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient"));
    }

    #[test]
    fn test_ledger_release_reservation() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let reservation_id = ledger.reserve(&alice_key(), 300).unwrap();
        assert_eq!(ledger.available_balance(&alice_key()).unwrap(), 700);

        ledger.release_reservation(reservation_id).unwrap();
        assert_eq!(ledger.available_balance(&alice_key()).unwrap(), 1000);
        assert_eq!(ledger.reserved_balance(&alice_key()).unwrap(), 0);
    }

    #[test]
    fn test_ledger_consume_reservation() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let reservation_id = ledger.reserve(&alice_key(), 300).unwrap();
        assert_eq!(ledger.balance(&alice_key()).unwrap(), 1000);

        ledger.consume_reservation(reservation_id).unwrap();

        // Balance is now reduced
        assert_eq!(ledger.balance(&alice_key()).unwrap(), 700);
        assert_eq!(ledger.reserved_balance(&alice_key()).unwrap(), 0);
        assert_eq!(ledger.available_balance(&alice_key()).unwrap(), 700);
    }

    #[test]
    fn test_ledger_double_release_fails() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let reservation_id = ledger.reserve(&alice_key(), 300).unwrap();
        ledger.release_reservation(reservation_id).unwrap();

        let result = ledger.release_reservation(reservation_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already released"));
    }

    #[test]
    fn test_ledger_transfer_with_reservation() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        // Reserve 600 credits
        ledger.reserve(&alice_key(), 600).unwrap();

        // Can transfer 400 (available balance)
        ledger.transfer(&alice_key(), &bob_key(), 400).unwrap();

        // Cannot transfer more than available
        let result = ledger.transfer(&alice_key(), &bob_key(), 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_ledger_multiple_reservations() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let res1 = ledger.reserve(&alice_key(), 200).unwrap();
        let res2 = ledger.reserve(&alice_key(), 300).unwrap();
        let res3 = ledger.reserve(&alice_key(), 100).unwrap();

        assert_eq!(ledger.reserved_balance(&alice_key()).unwrap(), 600);
        assert_eq!(ledger.available_balance(&alice_key()).unwrap(), 400);

        // Release one
        ledger.release_reservation(res2).unwrap();
        assert_eq!(ledger.reserved_balance(&alice_key()).unwrap(), 300);
        assert_eq!(ledger.available_balance(&alice_key()).unwrap(), 700);

        // Consume another
        ledger.consume_reservation(res1).unwrap();
        assert_eq!(ledger.balance(&alice_key()).unwrap(), 800);
        assert_eq!(ledger.reserved_balance(&alice_key()).unwrap(), 100);
        assert_eq!(ledger.available_balance(&alice_key()).unwrap(), 700);

        // Release the last one
        ledger.release_reservation(res3).unwrap();
        assert_eq!(ledger.balance(&alice_key()).unwrap(), 800);
        assert_eq!(ledger.available_balance(&alice_key()).unwrap(), 800);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HOST FUNCTION TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_host_credit_balance_with_capability() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let result = host_credit_balance(&caps, &ledger, &alice_key());
        assert!(result.success);
        assert!(result.error.is_none());

        let bytes = result.return_value.unwrap();
        assert_eq!(bytes.len(), 8);
        let balance = u64::from_le_bytes(bytes.try_into().unwrap());
        assert_eq!(balance, 1000);
    }

    #[test]
    fn test_host_credit_balance_without_capability() {
        let caps = CapabilitySet::new();
        let ledger = InMemoryCreditLedger::new();

        let result = host_credit_balance(&caps, &ledger, &alice_key());
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_credit_transfer_success() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let result = host_credit_transfer(&caps, &ledger, &alice_key(), &bob_key(), 400);
        assert!(result.success);
        assert!(result.error.is_none());

        assert_eq!(ledger.balance(&alice_key()).unwrap(), 600);
        assert_eq!(ledger.balance(&bob_key()).unwrap(), 400);
    }

    #[test]
    fn test_host_credit_transfer_without_capability() {
        let caps = CapabilitySet::new();
        let ledger = InMemoryCreditLedger::new();

        let result = host_credit_transfer(&caps, &ledger, &alice_key(), &bob_key(), 100);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_credit_transfer_zero_amount() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let result = host_credit_transfer(&caps, &ledger, &alice_key(), &bob_key(), 0);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("greater than zero"));
    }

    #[test]
    fn test_host_credit_transfer_same_account() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let result = host_credit_transfer(&caps, &ledger, &alice_key(), &alice_key(), 100);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("same account"));
    }

    #[test]
    fn test_host_credit_transfer_exceeds_max() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::new();

        let result = host_credit_transfer(
            &caps,
            &ledger,
            &alice_key(),
            &bob_key(),
            MAX_TRANSFER_AMOUNT + 1,
        );
        assert!(!result.success);
        assert!(result.error.unwrap().contains("exceeds maximum"));
    }

    #[test]
    fn test_host_credit_reserve_success() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let result = host_credit_reserve(&caps, &ledger, &alice_key(), 300);
        assert!(result.success);

        let bytes = result.return_value.unwrap();
        assert_eq!(bytes.len(), 8);
        let reservation_id = u64::from_le_bytes(bytes.try_into().unwrap());
        assert!(reservation_id > 0);

        assert_eq!(ledger.reserved_balance(&alice_key()).unwrap(), 300);
    }

    #[test]
    fn test_host_credit_reserve_without_capability() {
        let caps = CapabilitySet::new();
        let ledger = InMemoryCreditLedger::new();

        let result = host_credit_reserve(&caps, &ledger, &alice_key(), 100);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_credit_reserve_zero_amount() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        let result = host_credit_reserve(&caps, &ledger, &alice_key(), 0);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("greater than zero"));
    }

    #[test]
    fn test_host_credit_reserve_exceeds_max() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::new();

        let result = host_credit_reserve(&caps, &ledger, &alice_key(), MAX_RESERVE_AMOUNT + 1);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("exceeds maximum"));
    }

    #[test]
    fn test_host_credit_release_success() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        // First reserve
        let reserve_result = host_credit_reserve(&caps, &ledger, &alice_key(), 300);
        let bytes = reserve_result.return_value.unwrap();
        let reservation_id = u64::from_le_bytes(bytes.try_into().unwrap());

        // Then release
        let result = host_credit_release(&caps, &ledger, reservation_id);
        assert!(result.success);

        assert_eq!(ledger.available_balance(&alice_key()).unwrap(), 1000);
    }

    #[test]
    fn test_host_credit_consume_success() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        // First reserve
        let reserve_result = host_credit_reserve(&caps, &ledger, &alice_key(), 300);
        let bytes = reserve_result.return_value.unwrap();
        let reservation_id = u64::from_le_bytes(bytes.try_into().unwrap());

        // Then consume
        let result = host_credit_consume(&caps, &ledger, reservation_id);
        assert!(result.success);

        assert_eq!(ledger.balance(&alice_key()).unwrap(), 700);
    }

    #[test]
    fn test_host_credit_available_success() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        // Reserve some credits
        ledger.reserve(&alice_key(), 400).unwrap();

        let result = host_credit_available(&caps, &ledger, &alice_key());
        assert!(result.success);

        let bytes = result.return_value.unwrap();
        let available = u64::from_le_bytes(bytes.try_into().unwrap());
        assert_eq!(available, 600);
    }

    #[test]
    fn test_host_credit_with_unrestricted() {
        let caps = create_unrestricted_capset();
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);

        // All operations should work with unrestricted capability
        let balance_result = host_credit_balance(&caps, &ledger, &alice_key());
        assert!(balance_result.success);

        let transfer_result = host_credit_transfer(&caps, &ledger, &alice_key(), &bob_key(), 100);
        assert!(transfer_result.success);

        let reserve_result = host_credit_reserve(&caps, &ledger, &alice_key(), 100);
        assert!(reserve_result.success);

        let bytes = reserve_result.return_value.unwrap();
        let reservation_id = u64::from_le_bytes(bytes.try_into().unwrap());

        let release_result = host_credit_release(&caps, &ledger, reservation_id);
        assert!(release_result.success);

        let available_result = host_credit_available(&caps, &ledger, &alice_key());
        assert!(available_result.success);
    }

    #[test]
    fn test_ledger_clone_shares_state() {
        let ledger = InMemoryCreditLedger::with_balances(vec![(alice_key(), 1000)]);
        let ledger_clone = ledger.clone();

        // Modify through original
        ledger.transfer(&alice_key(), &bob_key(), 300).unwrap();

        // Clone sees the change
        assert_eq!(ledger_clone.balance(&alice_key()).unwrap(), 700);
        assert_eq!(ledger_clone.balance(&bob_key()).unwrap(), 300);
    }

    #[test]
    fn test_full_credit_workflow() {
        let caps = create_credit_caps();
        let ledger = InMemoryCreditLedger::new();

        // Fund Alice
        ledger.credit(&alice_key(), 10000).unwrap();

        // Check initial balance
        let result = host_credit_balance(&caps, &ledger, &alice_key());
        let balance = u64::from_le_bytes(result.return_value.unwrap().try_into().unwrap());
        assert_eq!(balance, 10000);

        // Transfer to Bob
        let result = host_credit_transfer(&caps, &ledger, &alice_key(), &bob_key(), 3000);
        assert!(result.success);

        // Reserve credits for an operation
        let result = host_credit_reserve(&caps, &ledger, &alice_key(), 2000);
        assert!(result.success);
        let reservation_id = u64::from_le_bytes(result.return_value.unwrap().try_into().unwrap());

        // Check available balance
        let result = host_credit_available(&caps, &ledger, &alice_key());
        let available = u64::from_le_bytes(result.return_value.unwrap().try_into().unwrap());
        assert_eq!(available, 5000); // 10000 - 3000 - 2000

        // Consume the reservation
        let result = host_credit_consume(&caps, &ledger, reservation_id);
        assert!(result.success);

        // Final balance check
        let result = host_credit_balance(&caps, &ledger, &alice_key());
        let balance = u64::from_le_bytes(result.return_value.unwrap().try_into().unwrap());
        assert_eq!(balance, 5000); // 10000 - 3000 - 2000

        let result = host_credit_balance(&caps, &ledger, &bob_key());
        let balance = u64::from_le_bytes(result.return_value.unwrap().try_into().unwrap());
        assert_eq!(balance, 3000);
    }
}
