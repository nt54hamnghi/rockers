use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use anyhow::Context;
use flate2::read::GzDecoder;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tar::Archive;
use tempfile::{self, TempDir};
use tokio::task::JoinSet;

use crate::cli::PullArgs;
use crate::image::ImageName;
use crate::registry::{ImageManifest, RegistryClient};

const TARGET: &str = "./tmp/rootfs";

impl PullArgs {
    pub async fn run(&self) -> anyhow::Result<()> {
        let image = self.image.parse::<ImageName>()?;
        let client = RegistryClient::new(image).await?;
        let ImageManifest { config, layers } = client.resolve_image_manifest().await?;

        let progress = MultiProgress::new();
        let style = ProgressStyle::with_template(
            "{msg:<2} [{bar:40.green/white}] {bytes:>8}/{total_bytes:8} ({bytes_per_sec}, {eta})",
        )?
        .progress_chars("=>-");

        fs::create_dir_all("./tmp")?;
        fs::create_dir_all(&TARGET)?;

        client
            .download_blob(&config, format!("./tmp/config.json"))
            .await?;

        let tmp_dir = TempDir::with_prefix("layers-")?;
        let mut paths = Vec::with_capacity(layers.len());
        let mut bars = Vec::with_capacity(layers.len());

        let mut set = JoinSet::new();
        for (index, layer) in layers.into_iter().enumerate() {
            // let path = format!("./tmp/layers/{}_{}.tar.gz", index, layer.digest);
            let path = tmp_dir
                .path()
                .join(format!("{}_{}.tar.gz", index, layer.digest));
            paths.push(path.clone());

            let bar = progress.add(ProgressBar::new(layer.size));
            bar.set_style(style.clone());
            bars.push(bar.clone());

            let client = client.clone();
            set.spawn(async move {
                let digest_short = &layer.digest[7..7 + 12];
                bar.set_message(format!("{digest_short}: Downloading"));
                let res = client
                    .download_blob_with_progress(&layer, &path, bar.clone())
                    .await;
                bar.finish_with_message(format!("{digest_short}: Download complete"));
                res
            });
        }

        while let Some(res) = set.join_next().await {
            res.unwrap()?;
        }

        for (path, bar) in paths.into_iter().zip(bars) {
            let digest_short = &path
                .file_name()
                .context("Path has no file name")?
                .to_str()
                .context("File name is not valid UTF-8")?
                .split(':')
                .last()
                .context("File name contains no ':' separator")?[7..7 + 12];

            let file = File::open(&path)?;

            bar.reset();
            bar.set_length(file.metadata()?.len());
            bar.set_style(style.clone());
            bar.set_message(format!("{digest_short}: Extracting"));

            extract_tar_gz(bar.wrap_read(file), &TARGET)?;

            bar.finish_with_message(format!("{digest_short}: Pull complete"));
        }

        Ok(())
    }
}

pub fn extract_tar_gz(file: impl Read, target_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.set_overwrite(true);
    archive.unpack(target_path)?;
    Ok(())
}
