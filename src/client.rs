use anyhow::{Context, Error, Result};
use fs_err as fs;
use fs_err::File;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use std::str;

/// Split into normal methods and inner so that the file cache can do some prefix trimming
/// while endpoints.yml doesn't need to go through that
pub trait Cache {
    fn get_inner(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn set_inner(&self, key: &str, data: &[u8]) -> Result<()>;
    fn get(&self, url: &str) -> Result<Option<Vec<u8>>> {
        self.get_inner(url)
    }
    fn set(&self, url: &str, data: &[u8]) -> Result<()> {
        self.set_inner(url, data)
    }
}

pub struct FileCache {
    cache_dir: PathBuf,
    prefix: String,
}

/// Stupid caching implementation that assumes that all urls share a common prefix
/// (which is currently the case for all implementations)
impl FileCache {
    pub fn new(cache_dir: PathBuf, prefix: String) -> Self {
        Self { cache_dir, prefix }
    }

    fn get_cache_file(&self, url: &str) -> Result<PathBuf> {
        let stripped_url = url.strip_prefix(&self.prefix).context(format!(
            "{} does not start with prefix {}",
            url, self.prefix
        ))?;
        let cache_file = self.cache_dir.join(stripped_url).with_extension("json");
        Ok(cache_file)
    }
}

impl Cache for FileCache {
    fn get_inner(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if PathBuf::from(key).is_file() {
            let mut file = File::open(key)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            Ok(Some(bytes))
        } else {
            Ok(None)
        }
    }

    fn set_inner(&self, key: &str, data: &[u8]) -> Result<()> {
        let path = PathBuf::from(key);
        let parent_dir = path
            .parent()
            .context(format!("No parent directory for {}", key))?;
        fs::create_dir_all(parent_dir)?;
        BufWriter::new(File::create(key)?).write_all(data)?;

        Ok(())
    }

    fn get(&self, url: &str) -> Result<Option<Vec<u8>>> {
        let cache_file = self.get_cache_file(url)?;
        self.get_inner(cache_file.to_str().unwrap())
    }

    fn set(&self, url: &str, data: &[u8]) -> Result<()> {
        let cache_file = self.get_cache_file(url)?;
        self.set_inner(cache_file.to_str().unwrap(), data)
    }
}

/// Noop cache
pub struct NoCache;

impl Cache for NoCache {
    fn get_inner(&self, _url: &str) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    fn set_inner(&self, _url: &str, _data: &[u8]) -> Result<()> {
        Ok(())
    }
}

/// Http client with oparl deserialization and caching
pub struct OparlClient<C: Cache> {
    pub cache: C,
    /// Actual reqwest client
    client: Client,
}

impl<C: Cache> OparlClient<C> {
    pub fn new(cache: C) -> Self {
        Self {
            cache,
            client: Client::new(),
        }
    }

    /// This function has to failure modes: The outer is for when the http/cache failed,
    /// the inner is for when deserialization into the schema failed and returns
    /// the raw serde value together with the error
    pub async fn get<T: 'static + DeserializeOwned + Send>(
        &self,
        url: &str,
    ) -> Result<Result<T, (Value, String)>> {
        if let Some(bytes) = self.cache.get(url)? {
            self.deserialize(bytes).await
        } else {
            let bytes = self.get_bytes(url).await?;
            self.cache.set(url, &bytes)?;

            self.deserialize(bytes).await
        }
    }

    async fn deserialize<T: 'static + DeserializeOwned + Send>(
        &self,
        bytes: Vec<u8>,
    ) -> Result<Result<T, (Value, String)>, Error> {
        let deserialize = move || match serde_json::from_slice(&bytes) {
            Ok(data) => Ok(Ok(data)),
            Err(err) => {
                let schemaless = serde_json::from_slice(&bytes)
                    .context("Es wurde ungültiges JSON zurückgegeben")?;
                Ok(Err((schemaless, err.to_string())))
            }
        };

        // I haven't found a way to spawn tasks on wasm, not sure if that's a platform restriction
        #[cfg(target_family = "wasm")]
        let data = deserialize();
        #[cfg(not(target_family = "wasm"))]
        let data = tokio::task::spawn_blocking(deserialize).await.unwrap();

        data
    }

    /// Bare uncached GET
    async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let get_builder = self.client.get(url);
        // We are not allowed to set the user agent in wasm
        #[cfg(not(target_family = "wasm"))]
        let get_builder = get_builder.header(
            "User-Agent",
            "oparl-validator/oparl-validator Konstantin Schütze <konstantin@schuetze.link>",
        );
        let response = get_builder
            .send()
            .await
            .context("API request failed to send")?;
        let response = response.error_for_status()?;
        let bytes = response
            .bytes()
            .await
            .context("API request failed to respond")?
            .to_vec();
        Ok(bytes)
    }
}
