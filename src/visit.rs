use crate::reporter::Reporter;
use crate::schema::{OparlUrl, OtherUrl};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Problem {
    pub kind: ProblemKind,
    pub path: String,
    pub detail: String,
}

impl Display for Problem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.path.is_empty() {
            write!(f, "{:?} {}: {}", self.kind, self.path, self.detail)
        } else {
            write!(f, "{:?}: {}", self.kind, self.detail)
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum ProblemKind {
    /// reqwest error (retries exceeded)
    HttpRequestFailedTooOften,
    /// reqwest error (retries not exceeded)
    HttpRequestFailedRetrying,
    /// An object that was expected to be in an external list was not part of any list
    /// and could not be fetched manually
    ObjectMissing(&'static str),
    /// An object that was expected to be in an external list was not part of any list,
    /// but could be fetched manually
    ObjectNotInList(&'static str),
    /// A field required by the oparl spec is missing
    RequiredFieldMissing,
    /// If a value is missing, either the field should be omitted or be `null`, not ""
    EmptyString,
    /// Any http url
    UsingHttp,
    /// serde error
    InvalidJson,
}

impl ProblemKind {
    pub fn message(&self, detail: &str) -> String {
        match self {
            ProblemKind::RequiredFieldMissing => {
                format!("Das zwingend vorgeschriebene Feld {} fehlt", detail)
            }
            ProblemKind::ObjectMissing(object_type) => {
                format!("Das Objekt vom Typ {} wurden von einem anderen Objekt verlinkt, ist aber nicht abrufbar", object_type)
            }
            ProblemKind::ObjectNotInList(object_type) => {
                format!("Das Objekt von Typ {} wurde von einem anderen Objekt verlinkt, fehlt aber in den externen Listen", object_type)
            }
            ProblemKind::EmptyString => {
                format!("Das Feld {} ist angegeben, hat aber keinen Inhalt", detail)
            }
            ProblemKind::UsingHttp => "Das unsichere HTTP wird verwendet".to_string(),
            ProblemKind::InvalidJson => "Es wurde kein gültiges JSON zurückgegeben".to_string(),
            ProblemKind::HttpRequestFailedRetrying => {
                "Die Abfrage schlug fehl und musste wiederholt werden".to_string()
            }
            ProblemKind::HttpRequestFailedTooOften => {
                "Die Abfrage schlug zu oft fehl und konnte nicht abgeschlossen werden".to_string()
            }
        }
    }
}

/// The impls are generated by the python script
pub trait OparlObject: Send + 'static {
    fn type_name() -> &'static str;
    fn visit_fields(&self, reporter: &impl Reporter, url: &str);
    fn get_required(&self) -> Vec<&str>;
    fn get_id(&self) -> Option<&str>;

    fn visit_field<T: Visitable>(
        &self,
        reporter: &impl Reporter,
        name: &str,
        field: &T,
        url: &str,
    ) {
        if self.get_required().contains(&name) && !field.is_some() {
            reporter.add_problem(Problem {
                detail: format_detail(self.get_id(), url),
                path: format!("{}.{}", Self::type_name(), name),
                kind: ProblemKind::RequiredFieldMissing,
            });
        }
        field.visit(
            reporter,
            url,
            self.get_id(),
            &format!("{}.{}", Self::type_name(), name),
        );
    }
}

/// Every struct, vec and field we can visit recursively  
pub trait Visitable {
    fn visit(&self, _reporter: &impl Reporter, _url: &str, _id: Option<&str>, _path: &str) {}

    /// We set all on optional to be more accepting, but we need this to check for required fields
    /// without `Any` or similar hacks
    fn is_some(&self) -> bool {
        true
    }
}

impl Visitable for usize {}
impl Visitable for bool {}
/// This one is for the location geojson
impl Visitable for HashMap<String, Value> {}

impl<T: OparlObject> Visitable for T {
    fn visit(&self, reporter: &impl Reporter, url: &str, _id: Option<&str>, _path: &str) {
        if let Some(id) = self.get_id() {
            reporter.add_id(id.deref().to_string());

            if id.starts_with("http:") {
                reporter.add_problem(Problem {
                    detail: format!("Das id Feld verwendet das unsichere http ({})", id),
                    path: String::new(),
                    kind: ProblemKind::UsingHttp,
                });
            }
        }
        self.visit_fields(reporter, url);
    }
}

impl<T: Visitable> Visitable for Vec<T> {
    fn visit(&self, reporter: &impl Reporter, url: &str, id: Option<&str>, path: &str) {
        for element in self {
            element.visit(reporter, url, id, path);
        }
    }
}

impl Visitable for String {
    fn visit(&self, reporter: &impl Reporter, url: &str, id: Option<&str>, path: &str) {
        if self.is_empty() {
            reporter.add_problem(Problem {
                detail: format_detail(id, url),
                path: path.to_string(),
                kind: ProblemKind::EmptyString,
            })
        }
    }
}

impl<T: OparlObject> Visitable for OparlUrl<T> {
    fn visit(&self, reporter: &impl Reporter, url: &str, id: Option<&str>, _path: &str) {
        reporter.add_link(
            T::type_name(),
            self.deref().clone(),
            id.unwrap_or(url).to_string(),
        );
    }
}

impl Visitable for OtherUrl {
    fn visit(&self, reporter: &impl Reporter, _url: &str, _id: Option<&str>, _path: &str) {
        reporter.add_other_url(self.deref().clone());
    }
}

impl<T: Visitable> Visitable for Option<T> {
    fn visit(&self, reporter: &impl Reporter, url: &str, id: Option<&str>, path: &str) {
        self.as_ref()
            .map(|x| x.visit(reporter, url, id, path))
            .unwrap_or_default()
    }

    fn is_some(&self) -> bool {
        self.is_some()
    }
}

pub fn format_detail(id: Option<&str>, url: &str) -> String {
    if id == Some(url) {
        url.to_string()
    } else {
        format!(
            "{} innerhalb von {}",
            id.unwrap_or("object with missing id"),
            url
        )
    }
}
