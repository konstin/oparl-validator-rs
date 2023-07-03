use crate::visit::Problem;
use crate::ProblemKind;
use parking_lot::Mutex;
use std::collections::{BTreeMap, HashMap, HashSet};

type TypeAndUrl = (&'static str, String);

#[derive(Default)]
pub struct Storage {
    pub problems: Mutex<Vec<Problem>>,
    pub known_ids: Mutex<HashSet<String>>,
    pub known_other_urls: Mutex<HashSet<String>>,
    pub linked_ids: Mutex<HashMap<TypeAndUrl, Vec<String>>>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            problems: Default::default(),
            known_ids: Default::default(),
            known_other_urls: Default::default(),
            linked_ids: Default::default(),
        }
    }

    pub fn add_problem(&self, problem: Problem) {
        self.problems.lock().push(problem);
    }

    /// Adds the id of an oparl objects (those we have seen)
    pub fn add_id(&self, id: String) {
        self.known_ids.lock().insert(id);
    }

    /// Adds a URL pointing to an oparl object (those we expect to see)
    pub fn add_link(&self, type_name: &'static str, target: String, source: String) {
        self.linked_ids
            .lock()
            .entry((type_name, target))
            .or_default()
            .push(source)
    }

    /// Adds a URL pointing to an external resource
    pub fn add_other_url(&self, url: String) {
        self.known_other_urls.lock().insert(url);
    }

    /// Returns the problems grouped by category for the report generation
    ///
    /// BTreeMap because it uses sorted keys so we get a sorted report later
    pub fn problems_grouped(&self) -> BTreeMap<(ProblemKind, String), Vec<Problem>> {
        let mut problem_groups = BTreeMap::new();
        for problem in self.problems.lock().iter() {
            problem_groups
                .entry((problem.kind.clone(), problem.path.clone()))
                .or_insert_with(Vec::new)
                .push(problem.clone())
        }
        problem_groups
    }
}
