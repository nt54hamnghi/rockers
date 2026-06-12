use serde::Deserialize;

pub const MANIFEST_LIST_V2: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
pub const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";
pub const MANIFEST_V2: &str = "application/vnd.docker.distribution.manifest.v2+json";
pub const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestList {
    pub manifests: Vec<ManifestDescriptor>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestDescriptor {
    pub digest: String,
    pub platform: Platform,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    pub architecture: String,
    pub os: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageManifest {
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub digest: String,
    pub size: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestEnvelope {
    pub media_type: Option<String>, // because OCI manifests don't always include the mediaType field.
                                    // We'll fall back to the Content-Type header
}
