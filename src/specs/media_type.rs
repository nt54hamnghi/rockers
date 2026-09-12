// https://github.com/opencontainers/image-spec/blob/main/media-types.md



use serde::{Deserialize, Serialize};

pub const OCI_DESCRIPTOR: &str = "application/vnd.oci.descriptor.v1+json";
pub const OCI_LAYOUT_HEADER: &str = "application/vnd.oci.layout.header.v1+json";
pub const OCI_IMAGE_INDEX: &str = "application/vnd.oci.image.index.v1+json";
pub const OCI_IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
pub const OCI_IMAGE_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
pub const OCI_IMAGE_LAYER_TAR: &str = "application/vnd.oci.image.layer.v1.tar";
pub const OCI_IMAGE_LAYER_TAR_GZIP: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
pub const OCI_IMAGE_LAYER_TAR_ZSTD: &str = "application/vnd.oci.image.layer.v1.tar+zstd";
pub const OCI_EMPTY: &str = "application/vnd.oci.empty.v1+json";

pub const DOCKER_DISTRIBUTION_MANIFEST_LIST: &str =
    "application/vnd.docker.distribution.manifest.list.v2+json";
pub const DOCKER_DISTRIBUTION_MANIFEST: &str =
    "application/vnd.docker.distribution.manifest.v2+json";
pub const DOCKER_CONTAINER_IMAGE: &str = "application/vnd.docker.container.image.v1+json";
pub const DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP: &str =
    "application/vnd.docker.image.rootfs.diff.tar.gzip";

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum MediaType {
    OCI_DESCRIPTOR,
    OCI_LAYOUT_HEADER,
    /// Compatible with [`DOCKER_DISTRIBUTION_MANIFEST_LIST`].
    OCI_IMAGE_INDEX,
    /// Compatible with [`DOCKER_DISTRIBUTION_MANIFEST`].
    OCI_IMAGE_MANIFEST,
    /// Compatible with [`DOCKER_CONTAINER_IMAGE`].
    OCI_IMAGE_CONFIG,
    OCI_IMAGE_LAYER_TAR,
    /// Compatible with [`DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP`].
    OCI_IMAGE_LAYER_TAR_GZIP,
    OCI_IMAGE_LAYER_TAR_ZSTD,
    OCI_EMPTY,
    /// Compatible with [`OCI_IMAGE_INDEX`].
    DOCKER_DISTRIBUTION_MANIFEST_LIST,
    /// Compatible with [`OCI_IMAGE_MANIFEST`].
    DOCKER_DISTRIBUTION_MANIFEST,
    /// Compatible with [`OCI_IMAGE_LAYER_TAR_GZIP`].
    DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP,
    /// Compatible with [`OCI_IMAGE_CONFIG`].
    DOCKER_CONTAINER_IMAGE,
    /// Unknown media type.
    // TODO: enforce RFC 6838, including the naming requirements in its section 4.2,
    // and MAY be registered with IANA.
    UNKNOWN(String),
}

impl From<MediaType> for String {
    fn from(media_type: MediaType) -> Self {
        let s = match &media_type {
            MediaType::OCI_DESCRIPTOR => OCI_DESCRIPTOR,
            MediaType::OCI_LAYOUT_HEADER => OCI_LAYOUT_HEADER,
            MediaType::OCI_IMAGE_INDEX => OCI_IMAGE_INDEX,
            MediaType::OCI_IMAGE_MANIFEST => OCI_IMAGE_MANIFEST,
            MediaType::OCI_IMAGE_CONFIG => OCI_IMAGE_CONFIG,
            MediaType::OCI_IMAGE_LAYER_TAR => OCI_IMAGE_LAYER_TAR,
            MediaType::OCI_IMAGE_LAYER_TAR_GZIP => OCI_IMAGE_LAYER_TAR_GZIP,
            MediaType::OCI_IMAGE_LAYER_TAR_ZSTD => OCI_IMAGE_LAYER_TAR_ZSTD,
            MediaType::OCI_EMPTY => OCI_EMPTY,
            MediaType::DOCKER_DISTRIBUTION_MANIFEST_LIST => DOCKER_DISTRIBUTION_MANIFEST_LIST,
            MediaType::DOCKER_DISTRIBUTION_MANIFEST => DOCKER_DISTRIBUTION_MANIFEST,
            MediaType::DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP => DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP,
            MediaType::DOCKER_CONTAINER_IMAGE => DOCKER_CONTAINER_IMAGE,
            MediaType::UNKNOWN(s) => s,
        };

        s.to_owned()
    }
}

impl From<String> for MediaType {
    fn from(media_type: String) -> Self {
        match media_type.as_str() {
            OCI_DESCRIPTOR => Self::OCI_DESCRIPTOR,
            OCI_LAYOUT_HEADER => Self::OCI_LAYOUT_HEADER,
            OCI_IMAGE_INDEX => Self::OCI_IMAGE_INDEX,
            OCI_IMAGE_MANIFEST => Self::OCI_IMAGE_MANIFEST,
            OCI_IMAGE_CONFIG => Self::OCI_IMAGE_CONFIG,
            OCI_IMAGE_LAYER_TAR => Self::OCI_IMAGE_LAYER_TAR,
            OCI_IMAGE_LAYER_TAR_GZIP => Self::OCI_IMAGE_LAYER_TAR_GZIP,
            OCI_IMAGE_LAYER_TAR_ZSTD => Self::OCI_IMAGE_LAYER_TAR_ZSTD,
            OCI_EMPTY => Self::OCI_EMPTY,
            DOCKER_DISTRIBUTION_MANIFEST_LIST => Self::DOCKER_DISTRIBUTION_MANIFEST_LIST,
            DOCKER_DISTRIBUTION_MANIFEST => Self::DOCKER_DISTRIBUTION_MANIFEST,
            DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP => Self::DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP,
            DOCKER_CONTAINER_IMAGE => Self::DOCKER_CONTAINER_IMAGE,
            _ => Self::UNKNOWN(media_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(OCI_DESCRIPTOR, MediaType::OCI_DESCRIPTOR)]
    #[case(OCI_LAYOUT_HEADER, MediaType::OCI_LAYOUT_HEADER)]
    #[case(OCI_IMAGE_INDEX, MediaType::OCI_IMAGE_INDEX)]
    #[case(OCI_IMAGE_MANIFEST, MediaType::OCI_IMAGE_MANIFEST)]
    #[case(OCI_IMAGE_CONFIG, MediaType::OCI_IMAGE_CONFIG)]
    #[case(OCI_IMAGE_LAYER_TAR, MediaType::OCI_IMAGE_LAYER_TAR)]
    #[case(OCI_IMAGE_LAYER_TAR_GZIP, MediaType::OCI_IMAGE_LAYER_TAR_GZIP)]
    #[case(OCI_IMAGE_LAYER_TAR_ZSTD, MediaType::OCI_IMAGE_LAYER_TAR_ZSTD)]
    #[case(OCI_EMPTY, MediaType::OCI_EMPTY)]
    fn from_accepts_oci_media_types(#[case] media_type: &str, #[case] expected: MediaType) {
        assert_eq!(MediaType::from(media_type.to_owned()), expected);
    }

    #[rstest]
    #[case(
        DOCKER_DISTRIBUTION_MANIFEST_LIST,
        MediaType::DOCKER_DISTRIBUTION_MANIFEST_LIST
    )]
    #[case(DOCKER_DISTRIBUTION_MANIFEST, MediaType::DOCKER_DISTRIBUTION_MANIFEST)]
    #[case(DOCKER_CONTAINER_IMAGE, MediaType::DOCKER_CONTAINER_IMAGE)]
    #[case(
        DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP,
        MediaType::DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP
    )]
    fn from_accepts_docker_media_types(#[case] media_type: &str, #[case] expected: MediaType) {
        assert_eq!(MediaType::from(media_type.to_string()), expected);
    }

    #[rstest]
    #[case(MediaType::OCI_DESCRIPTOR, OCI_DESCRIPTOR)]
    #[case(MediaType::OCI_LAYOUT_HEADER, OCI_LAYOUT_HEADER)]
    #[case(MediaType::OCI_IMAGE_INDEX, OCI_IMAGE_INDEX)]
    #[case(MediaType::OCI_IMAGE_MANIFEST, OCI_IMAGE_MANIFEST)]
    #[case(MediaType::OCI_IMAGE_CONFIG, OCI_IMAGE_CONFIG)]
    #[case(MediaType::OCI_IMAGE_LAYER_TAR, OCI_IMAGE_LAYER_TAR)]
    #[case(MediaType::OCI_IMAGE_LAYER_TAR_GZIP, OCI_IMAGE_LAYER_TAR_GZIP)]
    #[case(MediaType::OCI_IMAGE_LAYER_TAR_ZSTD, OCI_IMAGE_LAYER_TAR_ZSTD)]
    #[case(MediaType::OCI_EMPTY, OCI_EMPTY)]
    #[case(
        MediaType::DOCKER_DISTRIBUTION_MANIFEST_LIST,
        DOCKER_DISTRIBUTION_MANIFEST_LIST
    )]
    #[case(MediaType::DOCKER_DISTRIBUTION_MANIFEST, DOCKER_DISTRIBUTION_MANIFEST)]
    #[case(MediaType::DOCKER_CONTAINER_IMAGE, DOCKER_CONTAINER_IMAGE)]
    #[case(
        MediaType::DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP,
        DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP
    )]
    fn into_string_preserves_known_media_type_values(
        #[case] media_type: MediaType,
        #[case] expected: &str,
    ) {
        assert_eq!(String::from(media_type), expected);
    }

    #[test]
    fn from_preserves_unknown_media_type() {
        let media_type = "application/octet-stream";

        assert_eq!(
            MediaType::from(media_type.to_string()),
            MediaType::UNKNOWN(media_type.to_string())
        );
    }

    #[test]
    fn deserialize_preserves_unknown_media_type() {
        let media_type = "application/octet-stream";

        assert_eq!(
            serde_json::from_str::<MediaType>(&format!(r#""{media_type}""#)).unwrap(),
            MediaType::UNKNOWN(media_type.to_string())
        );
    }

    #[test]
    fn into_string_preserves_unknown_media_type() {
        let media_type = "application/octet-stream";

        assert_eq!(
            String::from(MediaType::UNKNOWN(media_type.to_owned())),
            media_type
        );
    }
}
