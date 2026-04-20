use std::path::Path;

use sha1::Digest;
use crate::models::ModVersion;

pub struct Downloader {
    client: reqwest::Client,
}

impl Downloader {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;
        Ok(Self { client })
    }

    pub async fn download_version(
        &self,
        version: &ModVersion,
        dest_dir: &Path,
    ) -> anyhow::Result<std::path::PathBuf> {
        let dest_path = dest_dir.join(&version.filename);

        if dest_path.exists() {
            if self.verify_hash(&dest_path, &version.hash_sha1, &version.hash_sha512)? {
                return Ok(dest_path);
            }
            std::fs::remove_file(&dest_path)?;
        }

        let resp = self
            .client
            .get(version.download_url.as_str())
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!(
                "Download failed: {} - {}",
                resp.status(),
                version.download_url
            );
        }

        let bytes = resp.bytes().await?;
        std::fs::write(&dest_path, &bytes)?;

        Ok(dest_path)
    }

    fn verify_hash(
        &self,
        path: &Path,
        expected_sha1: &Option<String>,
        expected_sha512: &Option<String>,
    ) -> anyhow::Result<bool> {
        let data = std::fs::read(path)?;

        if let Some(expected) = expected_sha1 {
            use sha1::Sha1;
            use std::io::Write;
            let mut hasher = Sha1::new();
            hasher.write_all(&data)?;
            let hash = format!("{:x}", hasher.finalize());
            return Ok(hash == expected.as_str());
        }

        if let Some(expected) = expected_sha512 {
            use sha2::Sha512;
            use std::io::Write;
            let mut hasher = Sha512::new();
            hasher.write_all(&data)?;
            let hash = format!("{:x}", hasher.finalize());
            return Ok(hash == expected.as_str());
        }

        Ok(true)
    }
}
