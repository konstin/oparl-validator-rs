use crate::reporter::Reporter;
use crate::visit::OparlObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;

/// Url linking to another oparl object
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OparlUrl<T>(String, PhantomData<T>);

impl<T> Deref for OparlUrl<T> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Url linking to a list or an external resource
#[derive(Debug, Serialize, Deserialize)]
pub struct OtherUrl(String);

impl Deref for OtherUrl {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub name: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub date: Option<String>,
    pub size: Option<usize>,
    pub sha1_checksum: Option<String>,
    pub text: Option<String>,
    pub access_url: Option<OtherUrl>,
    pub download_url: Option<OtherUrl>,
    pub external_service_url: Option<OtherUrl>,
    pub master_file: Option<OparlUrl<File>>,
    pub derivative_file: Option<Vec<OparlUrl<File>>>,
    pub file_license: Option<OtherUrl>,
    pub meeting: Option<Vec<OparlUrl<Meeting>>>,
    pub agenda_item: Option<Vec<OparlUrl<AgendaItem>>>,
    pub paper: Option<Vec<OparlUrl<Paper>>>,
    pub keyword: Option<Vec<String>>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub web: Option<OtherUrl>,
    pub deleted: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for File {
    fn type_name() -> &'static str {
        "File"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "name", &self.name, url);
        self.visit_field(reporter, "fileName", &self.file_name, url);
        self.visit_field(reporter, "mimeType", &self.mime_type, url);
        self.visit_field(reporter, "date", &self.date, url);
        self.visit_field(reporter, "size", &self.size, url);
        self.visit_field(reporter, "sha1Checksum", &self.sha1_checksum, url);
        self.visit_field(reporter, "text", &self.text, url);
        self.visit_field(reporter, "accessUrl", &self.access_url, url);
        self.visit_field(reporter, "downloadUrl", &self.download_url, url);
        self.visit_field(
            reporter,
            "externalServiceUrl",
            &self.external_service_url,
            url,
        );
        self.visit_field(reporter, "masterFile", &self.master_file, url);
        self.visit_field(reporter, "derivativeFile", &self.derivative_file, url);
        self.visit_field(reporter, "fileLicense", &self.file_license, url);
        self.visit_field(reporter, "meeting", &self.meeting, url);
        self.visit_field(reporter, "agendaItem", &self.agenda_item, url);
        self.visit_field(reporter, "paper", &self.paper, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "created", &self.created, url);
        self.visit_field(reporter, "modified", &self.modified, url);
        self.visit_field(reporter, "web", &self.web, url);
        self.visit_field(reporter, "deleted", &self.deleted, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type", "accessUrl"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct System {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub oparl_version: Option<String>,
    pub other_oparl_versions: Option<Vec<OparlUrl<System>>>,
    pub license: Option<OtherUrl>,
    pub body: Option<OtherUrl>,
    pub name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
    pub website: Option<OtherUrl>,
    pub vendor: Option<OtherUrl>,
    pub product: Option<OtherUrl>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub web: Option<OtherUrl>,
    pub deleted: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for System {
    fn type_name() -> &'static str {
        "System"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "oparlVersion", &self.oparl_version, url);
        self.visit_field(
            reporter,
            "otherOparlVersions",
            &self.other_oparl_versions,
            url,
        );
        self.visit_field(reporter, "license", &self.license, url);
        self.visit_field(reporter, "body", &self.body, url);
        self.visit_field(reporter, "name", &self.name, url);
        self.visit_field(reporter, "contactEmail", &self.contact_email, url);
        self.visit_field(reporter, "contactName", &self.contact_name, url);
        self.visit_field(reporter, "website", &self.website, url);
        self.visit_field(reporter, "vendor", &self.vendor, url);
        self.visit_field(reporter, "product", &self.product, url);
        self.visit_field(reporter, "created", &self.created, url);
        self.visit_field(reporter, "modified", &self.modified, url);
        self.visit_field(reporter, "web", &self.web, url);
        self.visit_field(reporter, "deleted", &self.deleted, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type", "oparlVersion", "body"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub body: Option<OparlUrl<Body>>,
    pub name: Option<String>,
    pub membership: Option<Vec<OparlUrl<Membership>>>,
    pub meeting: Option<OparlUrl<Meeting>>,
    pub short_name: Option<String>,
    pub post: Option<Vec<String>>,
    pub sub_organization_of: Option<OparlUrl<Organization>>,
    pub organization_type: Option<String>,
    pub classification: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub website: Option<OtherUrl>,
    pub location: Option<Location>,
    pub external_body: Option<OparlUrl<Body>>,
    pub keyword: Option<Vec<String>>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub web: Option<OtherUrl>,
    pub deleted: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for Organization {
    fn type_name() -> &'static str {
        "Organization"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "body", &self.body, url);
        self.visit_field(reporter, "name", &self.name, url);
        self.visit_field(reporter, "membership", &self.membership, url);
        self.visit_field(reporter, "meeting", &self.meeting, url);
        self.visit_field(reporter, "shortName", &self.short_name, url);
        self.visit_field(reporter, "post", &self.post, url);
        self.visit_field(
            reporter,
            "subOrganizationOf",
            &self.sub_organization_of,
            url,
        );
        self.visit_field(reporter, "organizationType", &self.organization_type, url);
        self.visit_field(reporter, "classification", &self.classification, url);
        self.visit_field(reporter, "startDate", &self.start_date, url);
        self.visit_field(reporter, "endDate", &self.end_date, url);
        self.visit_field(reporter, "website", &self.website, url);
        self.visit_field(reporter, "location", &self.location, url);
        self.visit_field(reporter, "externalBody", &self.external_body, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "created", &self.created, url);
        self.visit_field(reporter, "modified", &self.modified, url);
        self.visit_field(reporter, "web", &self.web, url);
        self.visit_field(reporter, "deleted", &self.deleted, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Consultation {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub paper: Option<OparlUrl<Paper>>,
    pub agenda_item: Option<OparlUrl<AgendaItem>>,
    pub meeting: Option<OparlUrl<Meeting>>,
    pub organization: Option<Vec<OparlUrl<Organization>>>,
    pub authoritative: Option<bool>,
    pub role: Option<String>,
    pub keyword: Option<Vec<String>>,
    pub web: Option<OtherUrl>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for Consultation {
    fn type_name() -> &'static str {
        "Consultation"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "paper", &self.paper, url);
        self.visit_field(reporter, "agendaItem", &self.agenda_item, url);
        self.visit_field(reporter, "meeting", &self.meeting, url);
        self.visit_field(reporter, "organization", &self.organization, url);
        self.visit_field(reporter, "authoritative", &self.authoritative, url);
        self.visit_field(reporter, "role", &self.role, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "web", &self.web, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paper {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub body: Option<OparlUrl<Body>>,
    pub name: Option<String>,
    pub reference: Option<String>,
    pub date: Option<String>,
    pub paper_type: Option<String>,
    pub related_paper: Option<Vec<OparlUrl<Paper>>>,
    pub superordinated_paper: Option<Vec<OparlUrl<Paper>>>,
    pub subordinated_paper: Option<Vec<OparlUrl<Paper>>>,
    pub main_file: Option<File>,
    pub auxiliary_file: Option<Vec<File>>,
    pub location: Option<Vec<Location>>,
    pub originator_person: Option<Vec<OparlUrl<Person>>>,
    pub under_direction_of: Option<Vec<OparlUrl<Organization>>>,
    pub originator_organization: Option<Vec<OparlUrl<Organization>>>,
    pub consultation: Option<Vec<Consultation>>,
    pub keyword: Option<Vec<String>>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub web: Option<OtherUrl>,
    pub deleted: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for Paper {
    fn type_name() -> &'static str {
        "Paper"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "body", &self.body, url);
        self.visit_field(reporter, "name", &self.name, url);
        self.visit_field(reporter, "reference", &self.reference, url);
        self.visit_field(reporter, "date", &self.date, url);
        self.visit_field(reporter, "paperType", &self.paper_type, url);
        self.visit_field(reporter, "relatedPaper", &self.related_paper, url);
        self.visit_field(
            reporter,
            "superordinatedPaper",
            &self.superordinated_paper,
            url,
        );
        self.visit_field(reporter, "subordinatedPaper", &self.subordinated_paper, url);
        self.visit_field(reporter, "mainFile", &self.main_file, url);
        self.visit_field(reporter, "auxiliaryFile", &self.auxiliary_file, url);
        self.visit_field(reporter, "location", &self.location, url);
        self.visit_field(reporter, "originatorPerson", &self.originator_person, url);
        self.visit_field(reporter, "underDirectionOf", &self.under_direction_of, url);
        self.visit_field(
            reporter,
            "originatorOrganization",
            &self.originator_organization,
            url,
        );
        self.visit_field(reporter, "consultation", &self.consultation, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "created", &self.created, url);
        self.visit_field(reporter, "modified", &self.modified, url);
        self.visit_field(reporter, "web", &self.web, url);
        self.visit_field(reporter, "deleted", &self.deleted, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegislativeTerm {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub body: Option<OparlUrl<Body>>,
    pub name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub keyword: Option<Vec<String>>,
    pub web: Option<OtherUrl>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for LegislativeTerm {
    fn type_name() -> &'static str {
        "LegislativeTerm"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "body", &self.body, url);
        self.visit_field(reporter, "name", &self.name, url);
        self.visit_field(reporter, "startDate", &self.start_date, url);
        self.visit_field(reporter, "endDate", &self.end_date, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "web", &self.web, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub system: Option<OparlUrl<System>>,
    pub short_name: Option<String>,
    pub name: Option<String>,
    pub website: Option<OtherUrl>,
    pub license: Option<OtherUrl>,
    pub license_valid_since: Option<String>,
    pub oparl_since: Option<String>,
    pub ags: Option<String>,
    pub rgs: Option<String>,
    pub equivalent: Option<Vec<OtherUrl>>,
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
    pub organization: Option<OtherUrl>,
    pub person: Option<OtherUrl>,
    pub meeting: Option<OtherUrl>,
    pub paper: Option<OtherUrl>,
    pub legislative_term: Option<Vec<LegislativeTerm>>,
    pub classification: Option<String>,
    pub location: Option<Location>,
    pub keyword: Option<Vec<String>>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub web: Option<OtherUrl>,
    pub deleted: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for Body {
    fn type_name() -> &'static str {
        "Body"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "system", &self.system, url);
        self.visit_field(reporter, "shortName", &self.short_name, url);
        self.visit_field(reporter, "name", &self.name, url);
        self.visit_field(reporter, "website", &self.website, url);
        self.visit_field(reporter, "license", &self.license, url);
        self.visit_field(
            reporter,
            "licenseValidSince",
            &self.license_valid_since,
            url,
        );
        self.visit_field(reporter, "oparlSince", &self.oparl_since, url);
        self.visit_field(reporter, "ags", &self.ags, url);
        self.visit_field(reporter, "rgs", &self.rgs, url);
        self.visit_field(reporter, "equivalent", &self.equivalent, url);
        self.visit_field(reporter, "contactEmail", &self.contact_email, url);
        self.visit_field(reporter, "contactName", &self.contact_name, url);
        self.visit_field(reporter, "organization", &self.organization, url);
        self.visit_field(reporter, "person", &self.person, url);
        self.visit_field(reporter, "meeting", &self.meeting, url);
        self.visit_field(reporter, "paper", &self.paper, url);
        self.visit_field(reporter, "legislativeTerm", &self.legislative_term, url);
        self.visit_field(reporter, "classification", &self.classification, url);
        self.visit_field(reporter, "location", &self.location, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "created", &self.created, url);
        self.visit_field(reporter, "modified", &self.modified, url);
        self.visit_field(reporter, "web", &self.web, url);
        self.visit_field(reporter, "deleted", &self.deleted, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec![
            "id",
            "type",
            "name",
            "organization",
            "person",
            "meeting",
            "paper",
            "legislativeTerm",
        ]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgendaItem {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub meeting: Option<OparlUrl<Meeting>>,
    pub number: Option<String>,
    pub name: Option<String>,
    pub public: Option<bool>,
    pub consultation: Option<OparlUrl<Consultation>>,
    pub result: Option<String>,
    pub resolution_text: Option<String>,
    pub resolution_file: Option<File>,
    pub auxiliary_file: Option<Vec<File>>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub keyword: Option<Vec<String>>,
    pub web: Option<OtherUrl>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for AgendaItem {
    fn type_name() -> &'static str {
        "AgendaItem"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "meeting", &self.meeting, url);
        self.visit_field(reporter, "number", &self.number, url);
        self.visit_field(reporter, "name", &self.name, url);
        self.visit_field(reporter, "public", &self.public, url);
        self.visit_field(reporter, "consultation", &self.consultation, url);
        self.visit_field(reporter, "result", &self.result, url);
        self.visit_field(reporter, "resolutionText", &self.resolution_text, url);
        self.visit_field(reporter, "resolutionFile", &self.resolution_file, url);
        self.visit_field(reporter, "auxiliaryFile", &self.auxiliary_file, url);
        self.visit_field(reporter, "start", &self.start, url);
        self.visit_field(reporter, "end", &self.end, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "web", &self.web, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meeting {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub name: Option<String>,
    pub meeting_state: Option<String>,
    pub cancelled: Option<bool>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub location: Option<Location>,
    pub organization: Option<Vec<OparlUrl<Organization>>>,
    pub participant: Option<Vec<OparlUrl<Person>>>,
    pub invitation: Option<File>,
    pub results_protocol: Option<File>,
    pub verbatim_protocol: Option<File>,
    pub auxiliary_file: Option<Vec<File>>,
    pub agenda_item: Option<Vec<AgendaItem>>,
    pub keyword: Option<Vec<String>>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub web: Option<OtherUrl>,
    pub deleted: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for Meeting {
    fn type_name() -> &'static str {
        "Meeting"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "name", &self.name, url);
        self.visit_field(reporter, "meetingState", &self.meeting_state, url);
        self.visit_field(reporter, "cancelled", &self.cancelled, url);
        self.visit_field(reporter, "start", &self.start, url);
        self.visit_field(reporter, "end", &self.end, url);
        self.visit_field(reporter, "location", &self.location, url);
        self.visit_field(reporter, "organization", &self.organization, url);
        self.visit_field(reporter, "participant", &self.participant, url);
        self.visit_field(reporter, "invitation", &self.invitation, url);
        self.visit_field(reporter, "resultsProtocol", &self.results_protocol, url);
        self.visit_field(reporter, "verbatimProtocol", &self.verbatim_protocol, url);
        self.visit_field(reporter, "auxiliaryFile", &self.auxiliary_file, url);
        self.visit_field(reporter, "agendaItem", &self.agenda_item, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "created", &self.created, url);
        self.visit_field(reporter, "modified", &self.modified, url);
        self.visit_field(reporter, "web", &self.web, url);
        self.visit_field(reporter, "deleted", &self.deleted, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub description: Option<String>,
    pub geojson: Option<HashMap<String, Value>>,
    pub street_address: Option<String>,
    pub room: Option<String>,
    pub postal_code: Option<String>,
    pub sub_locality: Option<String>,
    pub locality: Option<String>,
    pub bodies: Option<Vec<OparlUrl<Body>>>,
    pub organization: Option<Vec<OparlUrl<Organization>>>,
    pub meeting: Option<Vec<OparlUrl<Meeting>>>,
    pub papers: Option<Vec<OparlUrl<Paper>>>,
    pub keyword: Option<Vec<String>>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub web: Option<OtherUrl>,
    pub deleted: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for Location {
    fn type_name() -> &'static str {
        "Location"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "description", &self.description, url);
        self.visit_field(reporter, "geojson", &self.geojson, url);
        self.visit_field(reporter, "streetAddress", &self.street_address, url);
        self.visit_field(reporter, "room", &self.room, url);
        self.visit_field(reporter, "postalCode", &self.postal_code, url);
        self.visit_field(reporter, "subLocality", &self.sub_locality, url);
        self.visit_field(reporter, "locality", &self.locality, url);
        self.visit_field(reporter, "bodies", &self.bodies, url);
        self.visit_field(reporter, "organization", &self.organization, url);
        self.visit_field(reporter, "meeting", &self.meeting, url);
        self.visit_field(reporter, "papers", &self.papers, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "created", &self.created, url);
        self.visit_field(reporter, "modified", &self.modified, url);
        self.visit_field(reporter, "web", &self.web, url);
        self.visit_field(reporter, "deleted", &self.deleted, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub person: Option<OparlUrl<Person>>,
    pub organization: Option<OparlUrl<Organization>>,
    pub role: Option<String>,
    pub voting_right: Option<bool>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub on_behalf_of: Option<OparlUrl<Organization>>,
    pub keyword: Option<Vec<String>>,
    pub web: Option<OtherUrl>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for Membership {
    fn type_name() -> &'static str {
        "Membership"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "person", &self.person, url);
        self.visit_field(reporter, "organization", &self.organization, url);
        self.visit_field(reporter, "role", &self.role, url);
        self.visit_field(reporter, "votingRight", &self.voting_right, url);
        self.visit_field(reporter, "startDate", &self.start_date, url);
        self.visit_field(reporter, "endDate", &self.end_date, url);
        self.visit_field(reporter, "onBehalfOf", &self.on_behalf_of, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "web", &self.web, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub id: Option<OtherUrl>,
    pub r#type: Option<String>,
    pub body: Option<OparlUrl<Body>>,
    pub name: Option<String>,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub form_of_address: Option<String>,
    pub affix: Option<String>,
    pub title: Option<Vec<String>>,
    pub gender: Option<String>,
    pub phone: Option<Vec<String>>,
    pub email: Option<Vec<String>>,
    pub location: Option<OparlUrl<Location>>,
    pub status: Option<Vec<String>>,
    pub membership: Option<Vec<Membership>>,
    pub life: Option<String>,
    pub life_source: Option<String>,
    pub keyword: Option<Vec<String>>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub web: Option<OtherUrl>,
    pub deleted: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl OparlObject for Person {
    fn type_name() -> &'static str {
        "Person"
    }

    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {
        self.visit_field(reporter, "id", &self.id, url);
        self.visit_field(reporter, "type", &self.r#type, url);
        self.visit_field(reporter, "body", &self.body, url);
        self.visit_field(reporter, "name", &self.name, url);
        self.visit_field(reporter, "familyName", &self.family_name, url);
        self.visit_field(reporter, "givenName", &self.given_name, url);
        self.visit_field(reporter, "formOfAddress", &self.form_of_address, url);
        self.visit_field(reporter, "affix", &self.affix, url);
        self.visit_field(reporter, "title", &self.title, url);
        self.visit_field(reporter, "gender", &self.gender, url);
        self.visit_field(reporter, "phone", &self.phone, url);
        self.visit_field(reporter, "email", &self.email, url);
        self.visit_field(reporter, "location", &self.location, url);
        self.visit_field(reporter, "status", &self.status, url);
        self.visit_field(reporter, "membership", &self.membership, url);
        self.visit_field(reporter, "life", &self.life, url);
        self.visit_field(reporter, "lifeSource", &self.life_source, url);
        self.visit_field(reporter, "keyword", &self.keyword, url);
        self.visit_field(reporter, "created", &self.created, url);
        self.visit_field(reporter, "modified", &self.modified, url);
        self.visit_field(reporter, "web", &self.web, url);
        self.visit_field(reporter, "deleted", &self.deleted, url);
    }

    fn get_required(&self) -> Vec<&str> {
        vec!["id", "type"]
    }

    fn get_id(&self) -> Option<&str> {
        self.id.as_ref().map(|x| x.as_str())
    }
}
