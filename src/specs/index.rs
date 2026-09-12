// https://github.com/opencontainers/image-spec/blob/main/image-index.md

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::specs::{
    descriptor::Descriptor,
    media_type::{MediaType, OCI_IMAGE_INDEX},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageIndex {
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
    pub manifest: Vec<Descriptor>,
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
    // this field MUST contain `application/vnd.oci.image.index.v1+json` or compatible formats.
    if !matches!(
        &value,
        MediaType::OCI_IMAGE_INDEX | MediaType::DOCKER_DISTRIBUTION_MANIFEST_LIST
    ) {
        let msg = format!("mediaType must be {OCI_IMAGE_INDEX}, or compatible with it",);
        return Err(serde::de::Error::custom(msg));
    }

    Ok(Some(value))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::*;
    use crate::specs::media_type::{
        DOCKER_DISTRIBUTION_MANIFEST_LIST, OCI_IMAGE_CONFIG, OCI_IMAGE_MANIFEST,
    };

    impl Default for Descriptor {
        fn default() -> Self {
            serde_json::from_value(json!({
                "mediaType": OCI_IMAGE_MANIFEST,
                "size": 1392,
                "digest": "sha256:6fee4abc5fe6a21f40022199b2d5063eb591743c80d3d2f5fadf620a98cbae0d",
                "platform": {
                    "architecture": "amd64",
                    "os": "linux"
                },
                "annotations": {
                    "com.docker.official-images.bashbrew.arch": "amd64",
                    "org.opencontainers.image.base.name": "scratch",
                    "org.opencontainers.image.created": "2026-06-10T00:00:00Z",
                    "org.opencontainers.image.revision": "f45bb1d19b1c155a785d6a7d3e07c3fbb4221ccc",
                    "org.opencontainers.image.source": "https://git.launchpad.net/cloud-images/+oci/ubuntu-base",
                    "org.opencontainers.image.url": "https://hub.docker.com/_/ubuntu",
                    "org.opencontainers.image.version": "26.04"
                }
            }))
            .unwrap()
        }
    }

    #[rstest]
    fn deserialize_accepts_schema_version_2() {
        let json = json!({
            "schemaVersion": 2,
            "manifest": vec![Descriptor::default()],
        });

        let index = serde_json::from_value::<ImageIndex>(json).unwrap();

        assert_eq!(index.schema_version, 2);
    }

    #[rstest]
    #[case::oci_image_index(OCI_IMAGE_INDEX, MediaType::OCI_IMAGE_INDEX)]
    #[case::docker_distribution_manifest_list(
        DOCKER_DISTRIBUTION_MANIFEST_LIST,
        MediaType::DOCKER_DISTRIBUTION_MANIFEST_LIST
    )]
    fn deserialize_accepts_schema_version_2_with_image_index_media_type(
        #[case] media_type: &str,
        #[case] expected: MediaType,
    ) {
        let json = json!({
            "schemaVersion": 2,
            "mediaType": media_type,
            "manifest": vec![Descriptor::default()],
        });

        let index = serde_json::from_value::<ImageIndex>(json).unwrap();

        assert_eq!(index.schema_version, 2);
        assert_eq!(index.media_type, Some(expected));
    }

    #[rstest]
    fn deserialize_accepts_missing_media_type() {
        let json = json!({
            "schemaVersion": 2,
            "manifest": vec![Descriptor::default()],
        });

        let index = serde_json::from_value::<ImageIndex>(json).unwrap();

        assert_eq!(index.schema_version, 2);
        assert_eq!(index.media_type, None);
    }

    #[rstest]
    fn deserialize_accepts_optional_fields() {
        let json = json!({
            "schemaVersion": 2,
            "artifactType": "application/example",
            "manifest": vec![Descriptor::default()],
            "subject": Descriptor::default(),
            "annotations": {
                "org.opencontainers.image.title": "example"
            },
        });

        let index = serde_json::from_value::<ImageIndex>(json).unwrap();

        assert_eq!(index.artifact_type, Some("application/example".to_owned()));
        assert_eq!(index.manifest.len(), 1);
        assert!(index.subject.is_some());
        assert_eq!(
            index
                .annotations
                .unwrap()
                .get("org.opencontainers.image.title"),
            Some(&"example".to_owned())
        );
    }

    #[rstest]
    #[case::zero(0)]
    #[case::one(1)]
    #[case::three(3)]
    fn deserialize_rejects_schema_version_other_than_2(#[case] schema_version: u8) {
        let json = json!({
            "schemaVersion": schema_version,
            "manifest": vec![Descriptor::default()],
        });

        let err = serde_json::from_value::<ImageIndex>(json).unwrap_err();

        assert!(err.to_string().contains("schemaVersion must be 2"));
    }

    #[rstest]
    #[case::image_manifest(OCI_IMAGE_MANIFEST)]
    #[case::image_config(OCI_IMAGE_CONFIG)]
    #[case::unknown("application/octet-stream")]
    fn deserialize_rejects_non_index_media_type(#[case] media_type: &str) {
        let json = json!({
            "schemaVersion": 2,
            "mediaType": media_type,
            "manifest": vec![Descriptor::default()],
        });

        let err = serde_json::from_value::<ImageIndex>(json).unwrap_err();

        assert!(err.to_string().contains(
            "mediaType must be application/vnd.oci.image.index.v1+json, or compatible with it"
        ));
    }

    #[rstest]
    fn deserialize_rejects_missing_manifest() {
        let json = json!({
            "schemaVersion": 2,
        });

        let err = serde_json::from_value::<ImageIndex>(json).unwrap_err();

        assert!(err.to_string().contains("missing field `manifest`"));
    }
}
