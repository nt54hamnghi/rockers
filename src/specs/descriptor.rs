// https://github.com/opencontainers/image-spec/blob/main/descriptor.md

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::specs::media_type::MediaType;

static ALGORITHM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(?:[.+_-][a-z0-9]+)*$").unwrap());
static ENCODED_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9=_-]+$").unwrap());

/// Descriptor describes the disposition of the targeted content.
/// Its corresponding media type is `application/vnd.oci.descriptor.v1+json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub media_type: MediaType,
    pub digest: Digest,
    pub size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<Platform>,
}

const EMPTY_BLOB_DIGEST: &str =
    "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a";

impl Descriptor {
    pub fn empty() -> Self {
        let empty_blob_digest = Digest::try_from(EMPTY_BLOB_DIGEST.to_owned()).unwrap();

        Self {
            media_type: MediaType::OCI_EMPTY,
            digest: empty_blob_digest,
            size: 2,
            urls: None,
            annotations: None,
            data: None,
            artifact_type: None,
            platform: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        let empty_blob_digest = Digest::try_from(EMPTY_BLOB_DIGEST.to_owned()).unwrap();

        self.media_type == MediaType::OCI_EMPTY
            && self.digest == empty_blob_digest
            && self.size == 2
            && matches!(self.data.as_deref(), None | Some("e30="))
            && self.urls.is_none()
            && self.annotations.is_none()
            && self.artifact_type.is_none()
            && self.platform.is_none()
    }
}

/// Platform describes the minimum runtime requirements of
/// platform-specific images.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub architecture: String,
    pub os: String,
    #[serde(
        default,
        rename = "os.version",
        skip_serializing_if = "Option::is_none"
    )]
    pub os_version: Option<String>,
    #[serde(
        default,
        rename = "os.features",
        skip_serializing_if = "Option::is_none"
    )]
    pub os_features: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,
}

/// Digest acts as a content identifier, enabling content addressability.
/// It uniquely identifies content by taking a collision-resistant hash of the bytes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
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

impl From<Digest> for String {
    fn from(d: Digest) -> Self {
        d.to_string()
    }
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

impl Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = format!("{}:{}", self.algorithm, self.encoded);
        write!(f, "{}", d)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
            Self::Unregistered(_) => ENCODED_RE.is_match(encoded),
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
                if !ALGORITHM_RE.is_match(&value) {
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
    use serde_json::json;

    use super::*;

    #[test]
    fn descriptor_empty_returns_empty_blob_descriptor() {
        let descriptor = Descriptor::empty();

        assert_eq!(descriptor.media_type, MediaType::OCI_EMPTY);
        assert_eq!(descriptor.digest.to_string(), EMPTY_BLOB_DIGEST);
        assert_eq!(descriptor.size, 2);
        assert!(descriptor.urls.is_none());
        assert!(descriptor.annotations.is_none());
        assert!(descriptor.data.is_none());
        assert!(descriptor.artifact_type.is_none());
        assert!(descriptor.platform.is_none());
        assert!(descriptor.is_empty());
    }

    #[rstest]
    #[case::without_data(None)]
    #[case::with_empty_json_data(Some("e30=".to_owned()))]
    fn descriptor_is_empty_accepts_empty_blob_data_variants(#[case] data: Option<String>) {
        let mut descriptor = Descriptor::empty();
        descriptor.data = data;

        assert!(descriptor.is_empty());
    }

    #[test]
    fn digest_serializes_as_string() {
        let digest = Digest::try_from(EMPTY_BLOB_DIGEST.to_owned()).unwrap();

        assert_eq!(
            serde_json::to_value(digest).unwrap(),
            json!(EMPTY_BLOB_DIGEST)
        );
    }

    #[test]
    fn descriptor_serializes_required_fields() {
        let descriptor = Descriptor::empty();

        assert_eq!(
            serde_json::to_value(descriptor).unwrap(),
            json!({
                "mediaType": "application/vnd.oci.empty.v1+json",
                "digest": EMPTY_BLOB_DIGEST,
                "size": 2
            })
        );
    }

    #[test]
    fn descriptor_serialization_skips_none_optional_fields() {
        let descriptor = Descriptor::empty();

        assert_eq!(
            serde_json::to_value(descriptor).unwrap(),
            json!({
                "mediaType": "application/vnd.oci.empty.v1+json",
                "digest": EMPTY_BLOB_DIGEST,
                "size": 2
            })
        );
    }

    #[test]
    fn platform_serializes_renamed_fields_and_skips_none_optional_fields() {
        let platform = Platform {
            architecture: "amd64".to_owned(),
            os: "linux".to_owned(),
            os_version: Some("1.0".to_owned()),
            os_features: Some(vec!["sse4".to_owned()]),
            variant: None,
            features: None,
        };

        assert_eq!(
            serde_json::to_value(platform).unwrap(),
            json!({
                "architecture": "amd64",
                "os": "linux",
                "os.version": "1.0",
                "os.features": ["sse4"]
            })
        );
    }

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
    #[case::single_word("custom")]
    #[case::with_dot("custom.v1")]
    #[case::with_plus("custom+v1")]
    #[case::with_underscore("custom_v1")]
    #[case::with_dash("custom-v1")]
    fn algorithm_try_from_accepts_unregistered_algorithm_names(#[case] value: &str) {
        let algorithm = Algorithm::try_from(value.to_owned()).unwrap();

        assert_eq!(algorithm.to_string(), value);
    }

    #[rstest]
    #[case::empty("")]
    #[case::uppercase("Custom")]
    #[case::leading_separator(".custom")]
    #[case::trailing_separator("custom.")]
    #[case::double_separator("custom..v1")]
    fn algorithm_try_from_rejects_invalid_unregistered_algorithm_names(#[case] value: &str) {
        let err = Algorithm::try_from(value.to_owned()).unwrap_err();

        assert!(matches!(err, DigestParseError::InvalidAlgorithmFormat(_)));
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

    #[rstest]
    #[case::letters("abcDEF")]
    #[case::digits("123456")]
    #[case::underscore("abc_DEF")]
    #[case::dash("abc-DEF")]
    #[case::equals("abc=DEF")]
    fn unregistered_algorithm_validate_encoded_accepts_matching_encoded_re(#[case] encoded: &str) {
        let algorithm = Algorithm::Unregistered("custom".to_owned());

        assert!(algorithm.validate_encoded(encoded));
    }

    #[rstest]
    #[case::empty("")]
    #[case::dot("abc.def")]
    #[case::plus("abc+def")]
    #[case::slash("abc/def")]
    #[case::colon("abc:def")]
    fn unregistered_algorithm_validate_encoded_rejects_non_matching_encoded_re(
        #[case] encoded: &str,
    ) {
        let algorithm = Algorithm::Unregistered("custom".to_owned());

        assert!(!algorithm.validate_encoded(encoded));
    }
}
