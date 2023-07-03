//! Custom naive semaphore implementation for wasm
#[cfg(target_family = "wasm")]
use {anyhow::Result, parking_lot::Mutex};

#[cfg(not(target_family = "wasm"))]
pub use tokio::sync::Semaphore;

#[cfg(target_family = "wasm")]
pub struct Semaphore {
    inner: Mutex<u32>,
}

#[cfg(target_family = "wasm")]
impl Semaphore {
    pub fn new(inner: u32) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    pub async fn acquire(&self) -> Result<()> {
        *self.inner.lock() += 1;
        Ok(())
    }
}
