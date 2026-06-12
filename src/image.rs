use std::str::FromStr;

use anyhow::{Ok, anyhow};

pub enum Reference {
    Tag(String),
    Digest(String),
}

pub struct ImageName {
    pub repository: String,
    pub name: String,
    pub reference: Reference,
}

impl FromStr for ImageName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check for digest reference first: name@sha256...
        let (image_path, reference) = if let Some((path, digest)) = s.split_once('@') {
            (path, Reference::Digest(digest.to_string()))
        } else {
            // No '@' means it's a tag reference: name:tag or just name
            let (path, tag) = s.split_once(':').unwrap_or((s, "latest"));
            (path, Reference::Tag(tag.to_string()))
        };

        // Hanfle optional namespace
        let (repository, name) = if let Some((repo, name)) = image_path.split_once('/') {
            (repo.to_string(), name.to_string())
        } else {
            ("library".to_string(), image_path.to_string())
        };

        if repository.is_empty() || name.is_empty() {
            return Err(anyhow!(
                "Invalid image name. Expected format 'name:tag', 'name@digest', or 'repository/name:tag'"
            ));
        }

        Ok(ImageName {
            repository,
            name,
            reference,
        })
    }
}
