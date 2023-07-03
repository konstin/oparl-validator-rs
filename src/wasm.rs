#![cfg_attr(not(target_family = "wasm"), allow(clippy::unused_unit))]

use crate::client::{NoCache, OparlClient};
use crate::endpoints::EndpointsYmlEntry;
use crate::reporter::Reporter;
use crate::storage::Storage;
use crate::{validate_oparl_api, Cache, Problem, ProgressBarWrapper};
use anyhow::{format_err, Context, Result};
use parking_lot::Mutex;
use serde::Serialize;
use std::borrow::Cow;
use std::str;
use std::sync::Once;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{console, DomException, MessageChannel, MessagePort};

/// We communicate with wasm through a message channel
#[derive(Serialize)]
#[serde(tag = "message_type", rename_all = "kebab-case")]
pub enum WasmMessage {
    ProgressBarAdd {
        name: String,
    },
    ProgressBarUpdate {
        name: String,
        counter: String,
    },
    ProgressBarMessage {
        name: String,
        message: String,
    },
    ProgressBarFinish {
        name: String,
        /// We have "finish with message" from indicatif
        message: String,
    },
    ProblemReport {
        /// Whether this is from the end of a page or the actual is_final report
        is_final: bool,
        /// Preformatted strings to insert
        problems: Vec<String>,
    },
}

#[wasm_bindgen]
pub async fn test_city(city_or_url: String, callback: MessageChannel) -> Result<(), JsValue> {
    let callback = callback.port2();
    let reporter = ReporterWasm::new(Storage::new(), callback.clone());
    // Not using LocalStorage because of quota and old cache issue
    validate_oparl_api::<ReporterWasm, _>(&city_or_url, &reporter, &OparlClient::new(NoCache))
        .await
        .unwrap();
    console::log_1(&JsValue::from_str(&format!(
        "Found {} problems",
        reporter.get_storage().problems.lock().len()
    )));

    reporter.report_problems(true);

    Ok(())
}

#[wasm_bindgen]
pub async fn get_endpoints_js() -> Result<JsValue, JsValue> {
    let endpoints_yml =
        reqwest::get("https://raw.githubusercontent.com/OParl/resources/master/endpoints.yml")
            .await
            .map_err(|x| x.to_string())?
            .bytes()
            .await
            .map_err(|x| x.to_string())?;
    let endpoints: Vec<EndpointsYmlEntry> = serde_yaml::from_slice(&endpoints_yml).unwrap();
    // We can't return Vec<(String, String)> to wasm_bindgen due to
    // https://github.com/rustwasm/wasm-bindgen/issues/122
    // so we do the conversion ourselves
    let endpoint_tuples: Vec<(String, String)> = endpoints
        .into_iter()
        .map(|endpoint| (endpoint.title, endpoint.url))
        .collect();
    Ok(serde_wasm_bindgen::to_value(&endpoint_tuples).unwrap())
}

static START: Once = Once::new();

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    START.call_once(|| {
        console::log_1(&JsValue::from_str("wasm main called"));
    });

    Ok(())
}

/// Changes the web ui
pub struct ReporterWasm {
    callback: MessagePort,
    storage: Storage,
}

impl ReporterWasm {
    pub fn new(storage: Storage, callback: MessagePort) -> Self {
        Self { callback, storage }
    }

    pub fn report_problems(&self, is_final: bool) {
        let problem_groups = self.get_storage().problems_grouped();
        let problems = problem_groups
            .iter()
            .map(|(group, problems)| {
                format!(
                    "{} ({} Fälle). Beispiel: {}",
                    group.0.message(&group.1),
                    problems.len(),
                    // All vecs are non-empty
                    problems.iter().min().unwrap().detail
                )
            })
            .collect();

        self.callback
            .post_message(
                &serde_wasm_bindgen::to_value(&WasmMessage::ProblemReport { is_final, problems })
                    .unwrap(),
            )
            .expect("Failed to send problem report message");
    }
}

impl Reporter for ReporterWasm {
    type ProgressBar = ProgressBarWasm;

    fn get_storage(&self) -> &Storage {
        &self.storage
    }

    fn add_bar(&self, name: impl Into<Cow<'static, str>>) -> ProgressBarWasm {
        ProgressBarWasm::new(name.into().as_ref().to_string(), self.callback.clone())
            .map_err(|err| format_err!("{:?}", err))
            .context("Failed to render progress bar")
            .unwrap()
    }

    fn add_problem(&self, problem: Problem) {
        self.storage.add_problem(problem);
    }

    fn add_message(&self, message: &str) {
        // TODO
        console::log_1(&JsValue::from_str(&format!("Message: {}", message)));
    }

    fn finish_page(&self) {
        self.report_problems(false);
    }
}

impl ProgressBarWasm {
    fn new(name: String, callback: MessagePort) -> anyhow::Result<Self, JsValue> {
        let message = WasmMessage::ProgressBarAdd { name: name.clone() };
        callback
            .post_message(&serde_wasm_bindgen::to_value(&message).unwrap())
            .expect("Failed to send add message to js");
        Ok(Self {
            name,
            callback,
            count: Mutex::new(0),
            length: Mutex::new(None),
        })
    }
}

/// Currently not a real progress bar but a line with the information
pub struct ProgressBarWasm {
    name: String,
    callback: MessagePort,
    count: Mutex<u64>,
    length: Mutex<Option<u64>>,
}

impl ProgressBarWrapper for ProgressBarWasm {
    fn set_message(&self, message: impl Into<Cow<'static, str>>) {
        let message = WasmMessage::ProgressBarMessage {
            name: self.name.clone(),
            message: message.into().to_string(),
        };
        self.callback
            .post_message(&serde_wasm_bindgen::to_value(&message).unwrap())
            .expect("Failed to send message message to js");
    }

    fn inc(&self, value: u64) {
        // Avoid deadlocking
        let new_count = *self.count.lock() + value;
        *self.count.lock() = new_count;

        let text = if let Some(len) = *self.length.lock() {
            format!("{}/{}", new_count, len)
        } else {
            new_count.to_string()
        };
        let message = WasmMessage::ProgressBarUpdate {
            name: self.name.clone(),
            counter: text,
        };
        self.callback
            .post_message(&serde_wasm_bindgen::to_value(&message).unwrap())
            .expect("Failed to send update message to js");
    }

    fn set_length(&self, len: u64) {
        *self.length.lock() = Some(len);
        let text = if let Some(len) = *self.length.lock() {
            format!("{}/{}", self.count.lock(), len)
        } else {
            self.count.lock().to_string()
        };
        let message = WasmMessage::ProgressBarUpdate {
            name: self.name.clone(),
            counter: text,
        };
        self.callback
            .post_message(&serde_wasm_bindgen::to_value(&message).unwrap())
            .expect("Failed to send update message to js");
    }

    fn finish_with_message(&self, message: impl Into<Cow<'static, str>>) {
        let message = WasmMessage::ProgressBarFinish {
            name: self.name.clone(),
            message: message.into().to_string(),
        };
        self.callback
            .post_message(&serde_wasm_bindgen::to_value(&message).unwrap())
            .expect("Failed to send finish message to js");
    }

    fn println(&self, message: impl AsRef<str>) {
        console::log_1(&JsValue::from_str(message.as_ref()));
    }
}

/// Uses the browser's local storage. Note that local storage has a quota, so this becomes pretty useless
pub struct LocalStorage {
    local_storage: web_sys::Storage,
}

impl LocalStorage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        let local_storage = window
            .local_storage()
            .expect("no local storage exists")
            .expect("no local storage exists");
        Self { local_storage }
    }
}

impl Cache for LocalStorage {
    fn get_inner(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self
            .local_storage
            .get_item(key)
            .unwrap()
            .map(|x| x.as_bytes().to_vec()))
    }

    fn set_inner(&self, key: &str, data: &[u8]) -> Result<()> {
        let value = str::from_utf8(data).context("Can only handle utf8 data")?;
        match self.local_storage.set_item(key, value) {
            Ok(()) => Ok(()),
            Err(err) => match err.dyn_ref::<DomException>() {
                // We hit the storage quota
                Some(dom_exception) if dom_exception.code() == DomException::QUOTA_EXCEEDED_ERR => {
                    console::log_1(&JsValue::from_str(&format!(
                        "Local storage quota exhausted: {}",
                        dom_exception.message()
                    )));
                    Ok(())
                }
                _ => Err(format_err!("{:?}", err)),
            },
        }
    }
}
