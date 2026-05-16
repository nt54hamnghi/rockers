// https://github.com/opencontainers/image-spec/blob/main/media-types.md

use std::fmt::{self, Display};

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

const MEDIA_TYPES: &[&str] = &[
    OCI_DESCRIPTOR,
    OCI_LAYOUT_HEADER,
    OCI_IMAGE_INDEX,
    DOCKER_DISTRIBUTION_MANIFEST_LIST,
    OCI_IMAGE_MANIFEST,
    DOCKER_DISTRIBUTION_MANIFEST,
    OCI_IMAGE_CONFIG,
    DOCKER_CONTAINER_IMAGE,
    OCI_IMAGE_LAYER_TAR,
    OCI_IMAGE_LAYER_TAR_GZIP,
    DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP,
    OCI_IMAGE_LAYER_TAR_ZSTD,
    OCI_EMPTY,
];

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String")]
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
}

#[derive(Debug, thiserror::Error)]
pub struct UnknownMediaType(String);

impl Display for UnknownMediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown variant `{}`, expected one of {:?}",
            self.0, MEDIA_TYPES
        )
    }
}

impl TryFrom<String> for MediaType {
    type Error = UnknownMediaType;

    fn try_from(media_type: String) -> Result<Self, Self::Error> {
        match media_type.as_str() {
            OCI_DESCRIPTOR => Ok(Self::Descriptor),
            OCI_LAYOUT_HEADER => Ok(Self::LayoutHeader),
            OCI_IMAGE_INDEX | DOCKER_DISTRIBUTION_MANIFEST_LIST => Ok(Self::ImageIndex),
            OCI_IMAGE_MANIFEST | DOCKER_DISTRIBUTION_MANIFEST => Ok(Self::ImageManifest),
            OCI_IMAGE_CONFIG | DOCKER_CONTAINER_IMAGE => Ok(Self::ImageConfig),
            OCI_IMAGE_LAYER_TAR => Ok(Self::ImageLayerTar),
            OCI_IMAGE_LAYER_TAR_GZIP | DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP => {
                Ok(Self::ImageLayerTarGzip)
            }
            OCI_IMAGE_LAYER_TAR_ZSTD => Ok(Self::ImageLayerTarZstd),
            OCI_EMPTY => Ok(Self::Empty),
            _ => Err(UnknownMediaType(media_type)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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
    fn try_from_accepts_oci_media_types(#[case] media_type: &str, #[case] expected: MediaType) {
        assert_eq!(
            MediaType::try_from(media_type.to_string()).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case(DOCKER_DISTRIBUTION_MANIFEST_LIST, MediaType::ImageIndex)]
    #[case(DOCKER_DISTRIBUTION_MANIFEST, MediaType::ImageManifest)]
    #[case(DOCKER_CONTAINER_IMAGE, MediaType::ImageConfig)]
    #[case(DOCKER_IMAGE_ROOTFS_DIFF_TAR_GZIP, MediaType::ImageLayerTarGzip)]
    fn try_from_accepts_docker_compatible_media_types(
        #[case] media_type: &str,
        #[case] expected: MediaType,
    ) {
        assert_eq!(
            MediaType::try_from(media_type.to_string()).unwrap(),
            expected
        );
    }

    #[test]
    fn try_from_rejects_unknown_media_type() {
        let err = MediaType::try_from("application/octet-stream".to_string()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "unknown variant `application/octet-stream`, expected one of {:?}",
                MEDIA_TYPES
            )
        );
    }
}
