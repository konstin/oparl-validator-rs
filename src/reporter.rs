use crate::storage::Storage;
use crate::visit::Problem;
use std::borrow::Cow;

/// Handles the interaction between the actual validation logic and the CLI/UI.
///
/// This contains a storage, a way to generate progress bars and ways to add found problems and data
pub trait Reporter {
    type ProgressBar: ProgressBarWrapper;

    fn get_storage(&self) -> &Storage;

    /// Adds the id of an oparl objects (those we have seen)
    fn add_id(&self, id: String) {
        self.get_storage().add_id(id);
    }

    /// Adds a URL pointing to an oparl object (those we expect to see)
    fn add_link(&self, type_name: &'static str, target: String, source: String) {
        self.get_storage().add_link(type_name, target, source);
    }

    /// Adds a URL pointing to an external resource
    fn add_other_url(&self, url: String) {
        self.get_storage().add_other_url(url);
    }

    fn add_bar(&self, name: impl Into<Cow<'static, str>>) -> Self::ProgressBar;

    fn add_problem(&self, problem: Problem) {
        self.get_storage().add_problem(problem);
    }

    /// Add a string message to the log for the HEAD tester
    fn add_message(&self, message: &str);

    /// Add optional behavior after finishing a list page, in this case for wasm problem report update
    fn finish_page(&self) {}
}

//noinspection RsSelfConvention
pub trait ProgressBarWrapper {
    fn set_message(&self, message: impl Into<Cow<'static, str>>);
    fn inc(&self, value: u64);
    /// We might get the total length from the first page
    fn set_length(&self, value: u64);
    fn finish_with_message(&self, message: impl Into<Cow<'static, str>>);
    /// print when from a place with an associated progress bar
    fn println(&self, message: impl AsRef<str>);
}
