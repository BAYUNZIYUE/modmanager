use std::path::Path;

use sha1::Digest;
use crate::api::bmclapi::BmclapiMirror;
use crate::models::ModVersion;

pub struct Downloader {
    client: reqwest::Client,
    mirror: BmclapiMirror,
    use_mirror: bool,
}

impl Downloader {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;
        let mirror = BmclapiMirror::new()?;
        Ok(Self {
            client,
            mirror,
            use_mirror: false,
        })
    }

    pub fn with_mirror(mut self, enabled: bool) -> Self {
        self.use_mirror = enabled;
        self
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

        let download_url = self.resolve_url(version);

        let resp = self
            .client
            .get(&download_url)
            .send()
            .await?;

        if !resp.status().is_success() {
            if self.use_mirror && download_url == version.download_url.as_str() {
                if let Some(mirror_url) = BmclapiMirror::rewrite_download_url(&download_url) {
                    let retry_resp = self.client.get(&mirror_url).send().await?;
                    if retry_resp.status().is_success() {
                        let bytes = retry_resp.bytes().await?;
                        std::fs::write(&dest_path, &bytes)?;
                        return Ok(dest_path);
                    }
                }
            }
            anyhow::bail!("Download failed: {} - {}", resp.status(), download_url);
        }

        let bytes = resp.bytes().await?;
        std::fs::write(&dest_path, &bytes)?;

        Ok(dest_path)
    }

    fn resolve_url(&self, version: &ModVersion) -> String {
        let original = version.download_url.as_str().to_string();
        if self.use_mirror {
            BmclapiMirror::rewrite_download_url(&original).unwrap_or(original)
        } else {
            original
        }
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
            return Ok(hash.eq_ignore_ascii_case(expected));
        }

        if let Some(expected) = expected_sha512 {
            use sha2::Sha512;
            use std::io::Write;
            let mut hasher = Sha512::new();
            hasher.write_all(&data)?;
            let hash = format!("{:x}", hasher.finalize());
            return Ok(hash.eq_ignore_ascii_case(expected));
        }

        Ok(true)
    }
}
