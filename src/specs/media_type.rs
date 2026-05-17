// https://github.com/opencontainers/image-spec/blob/main/media-types.md

use serde::Deserialize;

pub const OCI_DESCRIPTOR: &str = "application/vnd.oci.descriptor.v1+json";
pub const OCI_LAYOUT_HEADER: &str = "application/vnd.oci.layout.header.v1+json";
/// Compatible with [`DOCKER_DISTRIBUTION_MANIFEST_LIST`].
pub const OCI_IMAGE_INDEX: &str = "application/vnd.oci.image.index.v1+json";
/// Compatible with [`DOCKER_DISTRIBUTION_MANIFEST`].
pub const OCI_IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
/// Compatible with [`DOCKER_CONTAINER_IMAGE`].
pub const OCI_IMAGE_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
pub const OCI_IMAGE_LAYER_TAR: &str = "application/vnd.oci.image.layer.v1.tar";
/// Compatible with [`DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP`].
pub const OCI_IMAGE_LAYER_TAR_GZIP: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
pub const OCI_IMAGE_LAYER_TAR_ZSTD: &str = "application/vnd.oci.image.layer.v1.tar+zstd";
pub const OCI_EMPTY: &str = "application/vnd.oci.empty.v1+json";

/// Compatible with [`OCI_IMAGE_INDEX`].
pub const DOCKER_DISTRIBUTION_MANIFEST_LIST: &str =
    "application/vnd.docker.distribution.manifest.list.v2+json";
/// Compatible with [`OCI_IMAGE_MANIFEST`].
pub const DOCKER_DISTRIBUTION_MANIFEST: &str =
    "application/vnd.docker.distribution.manifest.v2+json";
/// Compatible with [`OCI_IMAGE_LAYER_TAR_GZIP`].
pub const DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP: &str =
    "application/vnd.docker.image.rootfs.diff.tar.gzip";
/// Compatible with [`OCI_IMAGE_CONFIG`].
pub const DOCKER_CONTAINER_IMAGE: &str = "application/vnd.docker.container.image.v1+json";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(from = "String")]
pub enum MediaType {
    /// Content Descriptor.
    Descriptor,
    /// OCI Layout.
    LayoutHeader,
    /// Image Index.
    ImageIndex,
    /// Image manifest.
    ImageManifest,
    /// Image config.
    ImageConfig,
    /// Layer, as a tar archive.
    ImageLayerTar,
    /// Layer, as a tar archive compressed with gzip.
    ImageLayerTarGzip,
    /// Layer, as a tar archive compressed with zstd.
    ImageLayerTarZstd,
    /// Empty for unused descriptors.
    Empty,
    /// Unknown media type.
    Unknown(String),
}

impl From<String> for MediaType {
    fn from(media_type: String) -> Self {
        match media_type.as_str() {
            OCI_DESCRIPTOR => Self::Descriptor,
            OCI_LAYOUT_HEADER => Self::LayoutHeader,
            OCI_IMAGE_INDEX | DOCKER_DISTRIBUTION_MANIFEST_LIST => Self::ImageIndex,
            OCI_IMAGE_MANIFEST | DOCKER_DISTRIBUTION_MANIFEST => Self::ImageManifest,
            OCI_IMAGE_CONFIG | DOCKER_CONTAINER_IMAGE => Self::ImageConfig,
            OCI_IMAGE_LAYER_TAR => Self::ImageLayerTar,
            OCI_IMAGE_LAYER_TAR_GZIP | DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP => {
                Self::ImageLayerTarGzip
            }
            OCI_IMAGE_LAYER_TAR_ZSTD => Self::ImageLayerTarZstd,
            OCI_EMPTY => Self::Empty,
            _ => Self::Unknown(media_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(OCI_DESCRIPTOR, MediaType::Descriptor)]
    #[case(OCI_LAYOUT_HEADER, MediaType::LayoutHeader)]
    #[case(OCI_IMAGE_INDEX, MediaType::ImageIndex)]
    #[case(OCI_IMAGE_MANIFEST, MediaType::ImageManifest)]
    #[case(OCI_IMAGE_CONFIG, MediaType::ImageConfig)]
    #[case(OCI_IMAGE_LAYER_TAR, MediaType::ImageLayerTar)]
    #[case(OCI_IMAGE_LAYER_TAR_GZIP, MediaType::ImageLayerTarGzip)]
    #[case(OCI_IMAGE_LAYER_TAR_ZSTD, MediaType::ImageLayerTarZstd)]
    #[case(OCI_EMPTY, MediaType::Empty)]
    fn from_accepts_oci_media_types(#[case] media_type: &str, #[case] expected: MediaType) {
        assert_eq!(MediaType::from(media_type.to_string()), expected);
    }

    #[rstest]
    #[case(DOCKER_DISTRIBUTION_MANIFEST_LIST, MediaType::ImageIndex)]
    #[case(DOCKER_DISTRIBUTION_MANIFEST, MediaType::ImageManifest)]
    #[case(DOCKER_CONTAINER_IMAGE, MediaType::ImageConfig)]
    #[case(DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP, MediaType::ImageLayerTarGzip)]
    fn from_accepts_docker_compatible_media_types(
        #[case] media_type: &str,
        #[case] expected: MediaType,
    ) {
        assert_eq!(MediaType::from(media_type.to_string()), expected);
    }

    #[test]
    fn from_preserves_unknown_media_type() {
        let media_type = "application/octet-stream";

        assert_eq!(
            MediaType::from(media_type.to_string()),
            MediaType::Unknown(media_type.to_string())
        );
    }

    #[test]
    fn deserialize_preserves_unknown_media_type() {
        let media_type = "application/octet-stream";

        assert_eq!(
            serde_json::from_str::<MediaType>(&format!(r#""{media_type}""#)).unwrap(),
            MediaType::Unknown(media_type.to_string())
        );
    }
}
