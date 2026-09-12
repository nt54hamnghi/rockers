// https://github.com/opencontainers/image-spec/blob/main/manifest.md

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::specs::descriptor::Descriptor;
use crate::specs::media_type::{MediaType, OCI_IMAGE_MANIFEST};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u8,
    #[serde(
        default,
        deserialize_with = "deserialize_media_type",
        skip_serializing_if = "Option::is_none"
    )]
    pub media_type: Option<MediaType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<String>, // TODO: add constrains
    pub config: Descriptor,
    #[serde(deserialize_with = "deserialize_layers")]
    pub layers: Vec<Descriptor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<Descriptor>,
    // TODO: validate annotation keys and values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

fn deserialize_schema_version<'de, D>(d: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u8::deserialize(d)?;
    // The value must be 2 to ensure backward compatibility with older versions of Docker.
    if value != 2 {
        return Err(serde::de::Error::custom("schemaVersion must be 2"));
    }
    Ok(value)
}

fn deserialize_media_type<'de, D>(d: D) -> Result<Option<MediaType>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = MediaType::deserialize(d)?;
    // this field MUST contain `application/vnd.oci.image.manifest.v1+json` or compatible formats.
    if !matches!(
        &value,
        MediaType::OCI_IMAGE_MANIFEST | MediaType::DOCKER_DISTRIBUTION_MANIFEST
    ) {
        let msg = format!("mediaType must be {OCI_IMAGE_MANIFEST}, or compatible with it",);
        return Err(serde::de::Error::custom(msg));
    }

    Ok(Some(value))
}

fn deserialize_layers<'de, D>(d: D) -> Result<Vec<Descriptor>, D::Error>
where
    D: Deserializer<'de>,
{
    let layers = Vec::<Descriptor>::deserialize(d)?;
    if layers.is_empty() {
        return Err(serde::de::Error::custom(
            "layers must contain at least 1 item",
        ));
    }

    Ok(layers)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::*;
    use crate::specs::media_type::{
        DOCKER_DISTRIBUTION_MANIFEST, OCI_IMAGE_CONFIG, OCI_IMAGE_INDEX,
    };

    impl Default for Manifest {
        fn default() -> Self {
            Self {
                schema_version: 2,
                media_type: None,
                artifact_type: None,
                config: Descriptor::empty(),
                layers: vec![Descriptor::empty()],
                subject: None,
                annotations: None,
            }
        }
    }

    #[rstest]
    #[case::oci_image_manifest(OCI_IMAGE_MANIFEST, MediaType::OCI_IMAGE_MANIFEST)]
    #[case::docker_distribution_manifest(
        DOCKER_DISTRIBUTION_MANIFEST,
        MediaType::DOCKER_DISTRIBUTION_MANIFEST
    )]
    fn deserialize_accepts_schema_version_2_with_image_manifest_media_type(
        #[case] media_type: &str,
        #[case] expected: MediaType,
    ) {
        let json = json!({
            "schemaVersion": 2,
            "mediaType": media_type,
            "config": Descriptor::empty(),
            "layers": vec![Descriptor::empty()]
        });

        let manifest = serde_json::from_value::<Manifest>(json).unwrap();

        assert_eq!(manifest.schema_version, 2);
        assert_eq!(manifest.media_type, Some(expected));
    }

    #[rstest]
    fn deserialize_accepts_missing_media_type() {
        let json = json!({
            "schemaVersion": 2,
            "config": Descriptor::empty(),
            "layers": vec![Descriptor::empty()]
        });

        let manifest = serde_json::from_value::<Manifest>(json).unwrap();

        assert_eq!(manifest.schema_version, 2);
        assert_eq!(manifest.media_type, None);
    }

    #[rstest]
    #[case::zero(0)]
    #[case::one(1)]
    #[case::three(3)]
    fn deserialize_rejects_schema_version_other_than_2(#[case] schema_version: u8) {
        let json = json!({
            "schemaVersion": schema_version,
            "mediaType": OCI_IMAGE_MANIFEST,
            "config": Descriptor::empty(),
            "layers": vec![Descriptor::empty()]
        });

        let err = serde_json::from_value::<Manifest>(json).unwrap_err();

        assert!(err.to_string().contains("schemaVersion must be 2"));
    }

    #[rstest]
    #[case::image_index(OCI_IMAGE_INDEX)]
    #[case::image_config(OCI_IMAGE_CONFIG)]
    #[case::unknown("application/octet-stream")]
    fn deserialize_rejects_non_manifest_media_type(#[case] media_type: &str) {
        let json = json!({
            "schemaVersion": 2,
            "mediaType": media_type,
            "config": Descriptor::empty(),
            "layers": vec![Descriptor::empty()]
        });

        let err = serde_json::from_value::<Manifest>(json).unwrap_err();

        assert!(err.to_string().contains(
            "mediaType must be application/vnd.oci.image.manifest.v1+json, or compatible with it"
        ));
    }

    #[rstest]
    #[case::single(1)]
    #[case::multiple(2)]
    fn deserialize_accepts_at_least_one_layer(#[case] count: usize) {
        let json = json!({
            "schemaVersion": 2,
            "config": Descriptor::empty(),
            "layers": std::iter::repeat_n(Descriptor::empty(), count).collect::<Vec<_>>(),
        });

        let manifest = serde_json::from_value::<Manifest>(json).unwrap();

        assert_eq!(manifest.layers.len(), count);
    }

    #[rstest]
    fn deserialize_rejects_missing_layers() {
        let json = json!({
            "schemaVersion": 2,
            "config": Descriptor::empty(),
        });

        let err = serde_json::from_value::<Manifest>(json).unwrap_err();

        assert!(err.to_string().contains("missing field `layers`"));
    }

    #[rstest]
    fn deserialize_rejects_empty_layers() {
        let json = json!({
            "schemaVersion": 2,
            "config": Descriptor::empty(),
            "layers": []
        });

        let err = serde_json::from_value::<Manifest>(json).unwrap_err();

        assert!(
            err.to_string()
                .contains("layers must contain at least 1 item")
        );
    }
}
