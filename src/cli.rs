use crate::reporter::{ProgressBarWrapper, Reporter};
use crate::storage::Storage;
use fs_err::File;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use parking_lot::Mutex;
use std::borrow::Cow;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

pub struct ReporterCli {
    multi_progress: MultiProgress,
    storage: Storage,
    quiet: bool,
    log_file: Option<Arc<Mutex<File>>>,
}

impl ReporterCli {
    pub fn new(storage: Storage, quiet: bool, log_file: Option<File>) -> Self {
        Self {
            multi_progress: MultiProgress::new(),
            storage,
            quiet,
            log_file: log_file.map(|log_file| Arc::new(Mutex::new(log_file))),
        }
    }

    pub fn println(&self, message: &str) {
        if let Some(ref log_file) = self.log_file {
            writeln!(log_file.lock(), "{}", message).unwrap();
        }
    }
}

impl Reporter for ReporterCli {
    type ProgressBar = ProgressBarCli;

    fn get_storage(&self) -> &Storage {
        &self.storage
    }

    fn add_bar(&self, name: impl Into<Cow<'static, str>>) -> ProgressBarCli {
        let sty = ProgressStyle::default_spinner()
            .template("{spinner} [{elapsed_precise}] [{prefix}] Page {pos}: {msg}")
            .unwrap();

        let name = name.into();

        let progress_bar = if !self.quiet {
            let progress_bar = self.multi_progress.add(
                ProgressBar::new_spinner()
                    .with_style(sty)
                    .with_prefix(name.clone()),
            );
            progress_bar.enable_steady_tick(Duration::from_millis(100));
            progress_bar
        } else {
            ProgressBar::hidden()
        };

        ProgressBarCli::new(progress_bar, name, self.log_file.clone())
    }

    fn add_message(&self, message: &str) {
        if let Some(ref log_file) = self.log_file {
            writeln!(log_file.lock(), "{}", message).unwrap();
        }
    }
}

/// Currently prints to CLI and writes a hardcoded report.md
pub fn problem_report(
    storage: &Storage,
    out: &mut impl Write,
    report: &mut impl Write,
    is_final: bool,
) -> anyhow::Result<()> {
    let problem_groups = storage.problems_grouped();

    let summary_header = if is_final {
        "=== Validierungsreport: Zusammenfassung ==="
    } else {
        "=== ZWISCHENSTAND Zusammenfassung Validierungsreport ==="
    };
    writeln!(out, "{}", summary_header)?;
    writeln!(report, "{}", summary_header)?;
    for (group, problems) in problem_groups.iter() {
        let message = format!(
            "{} ({} Fälle). Beispiel: {}",
            group.0.message(&group.1),
            problems.len(),
            // All vecs are non-empty
            problems.iter().min().unwrap().detail
        );
        writeln!(out, "{}", message)?;
        writeln!(report, "{}", message)?;
    }

    Ok(())
}

pub fn write_detail_report(
    storage: &Storage,
    out: &mut impl Write,
    report: &mut impl Write,
) -> anyhow::Result<()> {
    let problem_groups = storage.problems_grouped();

    writeln!(report, "=== Detailreport: Alle Einzelfälle ===")?;
    for (group, problems) in problem_groups.iter() {
        writeln!(
            report,
            "== {} ({} Fälle) ==",
            group.0.message(&group.1),
            problems.len()
        )?;
        for problem in problems.iter().take(100) {
            writeln!(report, "{}", problem.detail)?;
        }
        if problems.len() > 100 {
            writeln!(report, "... und {} weitere Fälle", problems.len() - 100)?;
        }
    }
    writeln!(out, "\nAlle einzelnen Fälle finden sich in report.txt")?;
    Ok(())
}

pub struct ProgressBarCli {
    progress_bar: ProgressBar,
    name: String,
    log_file: Option<Arc<Mutex<File>>>,
}

impl ProgressBarCli {
    fn new(
        progress_bar: ProgressBar,
        name: impl Into<Cow<'static, str>>,
        log_file: Option<Arc<Mutex<File>>>,
    ) -> Self {
        Self {
            progress_bar,
            name: name.into().to_string(),
            log_file,
        }
    }
}

impl ProgressBarWrapper for ProgressBarCli {
    fn set_message(&self, message: impl Into<Cow<'static, str>>) {
        let message = message.into();
        self.progress_bar.set_message(message.clone());
        if let Some(ref log_file) = self.log_file {
            writeln!(
                log_file.lock(),
                "{} {}/{}: {}",
                self.name,
                self.progress_bar.position(),
                self.progress_bar.length().unwrap_or_default(),
                message
            )
            .unwrap();
        }
    }

    fn inc(&self, value: u64) {
        self.progress_bar.inc(value);
    }

    fn set_length(&self, len: u64) {
        let style = ProgressStyle::default_spinner()
            .template("{spinner} [{elapsed_precise}] [{prefix:<12}] Page {pos:>3}/{len:<3}: {msg}")
            .unwrap();
        self.progress_bar.set_style(style);
        self.progress_bar.set_length(len);
    }

    fn finish_with_message(&self, message: impl Into<Cow<'static, str>>) {
        let message = message.into();
        // Finish otherwise uses the position which is nonsensical for spinner
        self.progress_bar.set_length(self.progress_bar.position());
        self.progress_bar.finish_with_message(message.clone());

        if let Some(ref log_file) = self.log_file {
            writeln!(
                log_file.lock(),
                "{} {}/{} DONE: {}",
                self.name,
                self.progress_bar.position(),
                self.progress_bar.length().unwrap_or_default(),
                message
            )
            .unwrap();
        }
    }

    fn println(&self, message: impl AsRef<str>) {
        self.progress_bar.println(&message);
        if let Some(ref log_file) = self.log_file {
            writeln!(log_file.lock(), "{} {}", self.name, message.as_ref()).unwrap();
        }
    }
}

/// Just print each line to stdout because there are too many progress bars otherwise
pub struct ReporterBatch {
    storage: Storage,
    log_file: Arc<Mutex<File>>,
    /// The main log file used for all jobs together
    main_log_file: Arc<Mutex<File>>,
}

impl ReporterBatch {
    pub fn new(storage: Storage, log_file: File, main_log_file: Arc<Mutex<File>>) -> Self {
        Self {
            storage,
            log_file: Arc::new(Mutex::new(log_file)),
            main_log_file,
        }
    }

    pub fn println(&self, message: &str) {
        writeln!(self.log_file.lock(), "{}", message).unwrap();
    }
}

impl Reporter for ReporterBatch {
    type ProgressBar = ProgressBarBatch;

    fn get_storage(&self) -> &Storage {
        &self.storage
    }

    fn add_bar(&self, name: impl Into<Cow<'static, str>>) -> Self::ProgressBar {
        Self::ProgressBar::new(name, self.log_file.clone(), self.main_log_file.clone())
    }

    fn add_message(&self, message: &str) {
        writeln!(self.log_file.lock(), "{}", message).unwrap();
    }
}

pub struct ProgressBarBatch {
    name: String,
    count: Mutex<u64>,
    length: Mutex<Option<u64>>,
    log_file: Arc<Mutex<File>>,
    main_log_file: Arc<Mutex<File>>,
}

impl ProgressBarBatch {
    fn new(
        name: impl Into<Cow<'static, str>>,
        log_file: Arc<Mutex<File>>,
        main_log_file: Arc<Mutex<File>>,
    ) -> Self {
        Self {
            name: name.into().to_string(),
            count: Mutex::new(0),
            length: Mutex::new(None),
            log_file,
            main_log_file,
        }
    }
}

impl ProgressBarWrapper for ProgressBarBatch {
    fn set_message(&self, message: impl Into<Cow<'static, str>>) {
        writeln!(
            self.log_file.lock(),
            "{} {}/{:?}: {}",
            self.name,
            self.count.lock(),
            self.length.lock(),
            message.into()
        )
        .unwrap();
    }

    fn inc(&self, value: u64) {
        *self.count.lock() += value;
    }

    fn set_length(&self, value: u64) {
        *self.length.lock() = Some(value);
    }

    fn finish_with_message(&self, message: impl Into<Cow<'static, str>>) {
        writeln!(
            self.log_file.lock(),
            "{} {}/{:?} DONE: {}",
            self.name,
            self.count.lock(),
            self.length.lock(),
            message.into()
        )
        .unwrap();
    }

    fn println(&self, message: impl AsRef<str>) {
        writeln!(
            self.main_log_file.lock(),
            "{} {}",
            self.name,
            message.as_ref()
        )
        .unwrap();
    }
}
