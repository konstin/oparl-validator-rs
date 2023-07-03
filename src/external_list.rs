//! Serde types for external lists

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub total_elements: Option<usize>,
    pub elements_per_page: Option<usize>,
    pub current_page: Option<usize>,
    pub total_pages: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    pub next: Option<String>,
    pub first: Option<String>,
    #[serde(rename = "self")]
    pub self_: Option<String>,
    pub last: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalList<T> {
    pub data: Vec<T>,
    pub pagination: Pagination,
    pub links: Links,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct System {
    pub id: String,
    pub r#type: String,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}
