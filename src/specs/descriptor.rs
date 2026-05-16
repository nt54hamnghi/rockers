// https://github.com/opencontainers/image-spec/blob/main/descriptor.md

use std::collections::HashMap;
use std::fmt::Display;

use serde::Deserialize;

use crate::specs::media_type::MediaType;

/// Descriptor describes the disposition of the targeted content.
/// Its corresponding media type is `application/vnd.oci.descriptor.v1+json`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub media_type: MediaType,
    pub digest: Digest,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<String>,
}

/// Digest acts as a content identifier, enabling content addressability.
/// It uniquely identifies content by taking a collision-resistant hash of the bytes
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct Digest {
    /// The cryptographic hash function used to compute the digest.
    algorithm: Algorithm,
    /// The hex-encoded digest value.
    encoded: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DigestParseError {
    #[error("missing ':' in digest: {0}")]
    MissingSeparator(String),
    #[error("invalid algorithm format: {0}, expect matching ^[a-z0-9]+(?:[.+_-][a-z0-9]+)*$")]
    InvalidAlgorithmFormat(String),
    #[error("invalid encoded format: {0}, expect matching ^[a-zA-Z0-9=_-]+$")]
    InvalidEncodedFormat(String),
}

impl TryFrom<String> for Digest {
    type Error = DigestParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let Some((algo, enc)) = value.split_once(':') else {
            return Err(DigestParseError::MissingSeparator(value));
        };

        let algo = Algorithm::try_from(algo.to_owned())?;
        if !algo.validate_encoded(enc) {
            return Err(DigestParseError::InvalidEncodedFormat(enc.to_owned()));
        }

        Ok(Self {
            algorithm: algo,
            encoded: enc.to_owned(),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub enum Algorithm {
    #[default]
    Sha256,
    Sha512,
    Blake3,
    Unregistered(String),
}

fn is_hex_bytes(encoded: &str) -> bool {
    encoded.bytes().all(|b| b.is_ascii_hexdigit())
}

impl Algorithm {
    fn validate_encoded(&self, encoded: &str) -> bool {
        match self {
            Self::Sha256 => encoded.len() == 64 && is_hex_bytes(encoded),
            Self::Sha512 => encoded.len() == 128 && is_hex_bytes(encoded),
            Self::Blake3 => encoded.len() == 64 && is_hex_bytes(encoded),
            Self::Unregistered(s) => {
                // TODO: check with regex: ^[a-zA-Z0-9=_-]+$
                let matched = false;
                matched
            }
        }
    }
}

impl TryFrom<String> for Algorithm {
    type Error = DigestParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "sha256" => Ok(Self::Sha256),
            "sha512" => Ok(Self::Sha512),
            "blake3" => Ok(Self::Blake3),
            _ => {
                // TODO: check with regex: ^[a-z0-9]+(?:[.+_-][a-z0-9]+)*$
                let matched = false;
                if !matched {
                    return Err(DigestParseError::InvalidAlgorithmFormat(value));
                }
                Ok(Self::Unregistered(value))
            }
        }
    }
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sha256 => write!(f, "sha256"),
            Self::Sha512 => write!(f, "sha512"),
            Self::Blake3 => write!(f, "blake3"),
            Self::Unregistered(s) => write!(f, "{s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::sha256("sha256", Algorithm::Sha256)]
    #[case::sha512("sha512", Algorithm::Sha512)]
    #[case::blake3("blake3", Algorithm::Blake3)]
    fn algorithm_try_from_accepts_known_algorithms(
        #[case] value: &str,
        #[case] expected: Algorithm,
    ) {
        let algorithm = Algorithm::try_from(value.to_owned()).unwrap();

        assert_eq!(algorithm.to_string(), expected.to_string());
    }

    #[rstest]
    #[case::unknown("unknown")]
    fn algorithm_try_from_rejects_invalid_algorithm_format(#[case] value: &str) {
        // TODO
    }

    #[rstest]
    #[case::sha256(Algorithm::Sha256, 64)]
    #[case::sha512(Algorithm::Sha512, 128)]
    #[case::blake3(Algorithm::Blake3, 64)]
    fn algorithm_validate_encoded_accepts_hex_with_expected_length(
        #[case] algorithm: Algorithm,
        #[case] len: usize,
    ) {
        let encoded = "a".repeat(len);

        assert!(algorithm.validate_encoded(&encoded));
    }

    #[rstest]
    #[case::sha256(Algorithm::Sha256, 63)]
    #[case::sha512(Algorithm::Sha512, 127)]
    #[case::blake3(Algorithm::Blake3, 63)]
    fn algorithm_validate_encoded_rejects_wrong_length(
        #[case] algorithm: Algorithm,
        #[case] len: usize,
    ) {
        let encoded = "a".repeat(len);

        assert!(!algorithm.validate_encoded(&encoded));
    }

    #[rstest]
    #[case::sha256(Algorithm::Sha256, 64)]
    #[case::sha512(Algorithm::Sha512, 128)]
    #[case::blake3(Algorithm::Blake3, 64)]
    fn algorithm_validate_encoded_rejects_non_hex(
        #[case] algorithm: Algorithm,
        #[case] len: usize,
    ) {
        let encoded = "g".repeat(len);

        assert!(!algorithm.validate_encoded(&encoded));
    }
}
