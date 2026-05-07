use std::env::consts;
use std::path::Path;

use anyhow::{Context, Ok};
use indicatif::ProgressBar;
use reqwest::header::{self, HeaderMap};
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use crate::image::{ImageName, Reference};
use crate::registry::types::{Descriptor, ImageManifest, ManifestEnvelope, ManifestList};
use crate::registry::{MANIFEST_LIST_V2, MANIFEST_V2, OCI_INDEX, OCI_MANIFEST};

const AUTH_URL: &str = "https://auth.docker.io";
const REGISTRY_URL: &str = "https://registry-1.docker.io";

#[derive(Debug, Clone)]
pub struct RegistryClient {
    client: reqwest::Client,
    repository: String,
    reference: String,
}

impl RegistryClient {
    pub async fn new(image: ImageName) -> anyhow::Result<Self> {
        let repository = format!("{}/{}", image.repository, image.name);
        let reference = match image.reference {
            Reference::Tag(t) => t,
            Reference::Digest(d) => d,
        };

        #[derive(Deserialize)]
        struct TokenResponse {
            token: String,
        }

        let token = Client::new()
            .get(format!("{AUTH_URL}/token"))
            .query(&[("service", "registry.docker.io")])
            .query(&[("scope", format!("repository:{repository}:pull"))])
            .send()
            .await?
            .json::<TokenResponse>()
            .await?
            .token;

        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            format!("Bearer {}", token).parse().unwrap(),
        );
        let client = ClientBuilder::new().default_headers(headers).build()?;

        Ok(Self {
            client,
            repository,
            reference,
        })
    }

    pub async fn get_manifest_response(
        &self,
        reference: &str,
        accept: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let repository = &self.repository;
        let url = format!("{REGISTRY_URL}/v2/{repository}/manifests/{reference}");

        let res = self
            .client
            .get(url)
            .header(header::ACCEPT, accept)
            .send()
            .await?
            .error_for_status()?;

        Ok(res)
    }

    pub async fn resolve_image_manifest(&self) -> anyhow::Result<ImageManifest> {
        let os = consts::OS;
        let arch = if consts::ARCH == "x86_64" {
            "amd64"
        } else {
            consts::ARCH
        };

        // Step 1 - ONE request, accepting both shapes
        let accept = format!("{MANIFEST_LIST_V2}, {OCI_INDEX},  {MANIFEST_V2}, {OCI_MANIFEST}");
        let response = self.get_manifest_response(&self.reference, &accept).await?;

        // Step 2 - save Content-Type header before consuming the response body
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        // Read raw bytes - we'll deserialize them ourselves instead of using .json()
        let bytes = response.bytes().await?;

        // Peak at the mediaType field in the JSON, fall back to Content-Type header
        let envelope: ManifestEnvelope = serde_json::from_slice(&bytes)?;
        let media_type = envelope.media_type.as_deref().unwrap_or(&content_type);

        match media_type {
            // Step 3 - Got a manifest list? Find our platform, make a second call
            MANIFEST_LIST_V2 | OCI_INDEX => {
                let list: ManifestList = serde_json::from_slice(&bytes)?;
                let desc = list
                    .manifests
                    .into_iter()
                    .find(|m| m.platform.architecture == arch && m.platform.os == os)
                    .with_context(|| {
                        format!(
                            "No manifest found for the current platform (os: {os}, arch: {arch})"
                        )
                    })?;
                self.get_image_manifest(&desc.digest).await
            }
            // Step 4 - Got an image manifest directly? Already done
            _ => Ok(serde_json::from_slice(&bytes)?),
        }
    }

    async fn get_image_manifest(&self, reference: &str) -> anyhow::Result<ImageManifest> {
        let accept = "application/vnd.docker.distribution.manifest.v2+json, application/vnd.oci.image.manifest.v1+json";

        let res = self
            .get_manifest_response(reference, accept)
            .await?
            .json::<ImageManifest>()
            .await?;

        Ok(res)
    }

    pub async fn download_blob(
        &self,
        layer: &Descriptor,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let repository = &self.repository;
        let digest = &layer.digest;

        let url = format!("{REGISTRY_URL}/v2/{repository}/blobs/{digest}");
        let resp = self.client.get(url).send().await?;
        let bytes = resp.bytes().await?;
        tokio::fs::write(path, bytes).await?;

        Ok(())
    }

    pub async fn download_blob_with_progress(
        &self,
        layer: &Descriptor,
        path: impl AsRef<Path>,
        bar: ProgressBar,
    ) -> anyhow::Result<()> {
        let repository = &self.repository;
        let digest = &layer.digest;

        let url = format!("{REGISTRY_URL}/v2/{repository}/blobs/{digest}");
        let mut resp = self.client.get(url).send().await?;

        let mut file = tokio::fs::File::create(&path).await?;
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
            bar.inc(chunk.len() as u64);
        }
        file.flush().await?;

        Ok(())
    }
}
