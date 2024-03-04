use std::env;

use crate::error::TxError;

/// This module provides mocked cryptographic functions.
/// TODO: Remove when the actual `kairos-crypto` package becomes available.
///

/// Signs the given data using a mock mechanism.
pub fn sign(data: &[u8], _private_key: &[u8], public_key: &[u8]) -> Vec<u8> {
    // Note: Private key is unused in the mock implementation but included for interface compatibility.
    // Mock signature combines data and public_key for simplicity.
    [data, public_key].concat()
}

// Name of ENV variable that can disable signature check.
// NOTE: This is useful for testing purposes.
pub const SKIP_SIGNATURE_ENV_VAR: &str = "KAIROS_SKIP_SIGNATURE";

/// Verifies a mock signature against the given data and public key.
/// If the ENV variable "KAIROS_SKIP_SIGNATURE" is set to "true",
/// the verification step will be skipped.
pub fn verify(data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<(), TxError> {
    // Skip verification if disabled by the environment variable.
    if env::var(SKIP_SIGNATURE_ENV_VAR)
        .map(|v| v == "true")
        .unwrap_or(false)
    {
        return Ok(());
    }

    // Re-construct the expected signature using the mock `sign` function logic
    let expected_signature = sign(data, &[], public_key);

    // Check if the provided signature matches the expected signature
    if signature != expected_signature.as_slice() {
        return Err(TxError::InvalidSignature);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify_success() {
        let data = b"test data";
        let private_key = b"private_key";
        let public_key = b"public_key";

        let signature = sign(data, private_key, public_key);

        // This should succeed since we're using the correct public key
        assert!(verify(data, &signature, public_key).is_ok());
    }

    #[test]
    fn test_verify_failure_due_to_incorrect_public_key() {
        let data = b"test data";
        let private_key = b"private_key";
        let public_key = b"public_key";
        let incorrect_public_key = b"incorrect_public_key";

        let signature = sign(data, private_key, public_key);

        // This should fail since the public key does not match
        assert!(verify(data, &signature, incorrect_public_key).is_err());
    }
}
