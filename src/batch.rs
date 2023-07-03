#![cfg_attr(target_family = "wasm", allow(unused_imports))]
use crate::cli::{problem_report, write_detail_report, ReporterBatch};
use crate::client::{FileCache, NoCache};
use crate::endpoints::get_endpoints;
use crate::storage::Storage;
use crate::{validate_oparl_api, OparlClient, Reporter};
use anyhow::bail;
use fs_err as fs;
use fs_err::File;
use parking_lot::Mutex;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use slug::slugify;
use std::io::BufWriter;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Runs the validation for a single city in the batch job
/// with optional caching, finally writing the reports
pub async fn validate_oparl_api_cli(
    name: &str,
    entrypoint: &str,
    cache_dir: &Option<PathBuf>,
    reporter: &impl Reporter,
    report: PathBuf,
    main_log: Arc<Mutex<File>>,
) -> anyhow::Result<()> {
    if let Some(cache_dir) = cache_dir.clone() {
        let prefix = entrypoint.trim_end_matches("system").to_string();
        let cache = FileCache::new(cache_dir.join(name), prefix);
        validate_oparl_api::<_, _>(entrypoint, reporter, &OparlClient::new(cache)).await?;
    } else {
        validate_oparl_api::<_, _>(entrypoint, reporter, &OparlClient::new(NoCache)).await?;
    }

    let mut report = BufWriter::new(File::create(report)?);
    problem_report(
        reporter.get_storage(),
        &mut *main_log.lock(),
        &mut report,
        true,
    )?;
    write_detail_report(reporter.get_storage(), &mut *main_log.lock(), &mut report)?;

    Ok(())
}

#[cfg(not(target_family = "wasm"))]
fn validate_all_entry(
    title: &str,
    url: &str,
    cache_dir: &Option<PathBuf>,
    report: &Path,
    log: &Path,
    main_log: Arc<Mutex<File>>,
) -> anyhow::Result<()> {
    let log_file = log.join(format!("{}.txt", slugify(title)));
    let reporter = ReporterBatch::new(Storage::new(), File::create(log_file)?, main_log.clone());
    let report = report.join(format!("{}.txt", slugify(title)));
    reporter.println("START");

    let rt = tokio::runtime::Runtime::new()?;
    let result: anyhow::Result<()> = rt.block_on(validate_oparl_api_cli(
        title, url, cache_dir, &reporter, report, main_log,
    ));

    match result {
        Ok(()) => {
            reporter.println("DONE");
            Ok(())
        }
        Err(err) => {
            reporter.println(&format!("FAILED: {}", err));
            for cause in err.chain().collect::<Vec<_>>().iter() {
                reporter.println(&format!("  Caused by: {}", cause));
            }
            Err(err)
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub async fn validate_all(
    cache_dir: &Option<PathBuf>,
    report: &Path,
    log: &Path,
    quiet: bool,
) -> anyhow::Result<()> {
    if quiet {
        bail!("Unsupported quiet");
    }

    fs::create_dir_all(report)?;
    fs::create_dir_all(log)?;
    let endpoints = get_endpoints(&NoCache).await?;

    let endpoints: Vec<(&str, &str)> = endpoints
        .iter()
        // Skip the oparl mirror because it is large an pointless to validate
        .filter(|endpoint| endpoint.url != "https://mirror.oparl.org/system")
        .map(|endpoint| (endpoint.title.as_str(), endpoint.url.as_str()))
        .collect();

    rayon::ThreadPoolBuilder::new()
        .num_threads(endpoints.len())
        .build_global()
        .unwrap();

    let main_log = File::create(log.join("0_oparl-validator-rs.txt"))?;
    let main_log = Arc::new(Mutex::new(main_log));

    writeln!(
        main_log.lock(),
        "Started {} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )?;
    let results: Vec<anyhow::Result<()>> = endpoints
        .par_iter()
        .map(|(title, url)| {
            writeln!(main_log.lock(), "Started {} ({})", title, url)?;
            let start = Instant::now();
            let result = validate_all_entry(title, url, cache_dir, report, log, main_log.clone());
            let end = Instant::now();
            match result {
                Ok(()) => {
                    writeln!(
                        main_log.lock(),
                        "DONE {} in {}s ({})",
                        title,
                        (end - start).as_secs(),
                        url
                    )?;
                    Ok(())
                }
                Err(err) => {
                    // Write the error string all at once so it's one block and doesn't get interleaved by other threads
                    let mut err_string = String::new();
                    err_string += &format!("FAILED {} ({})", title, url);
                    for cause in err.chain().collect::<Vec<_>>().iter() {
                        err_string += &format!("\n  Caused by: {}", cause);
                    }
                    writeln!(main_log.lock(), "{}", err_string)?;
                    Err(err)
                }
            }
        })
        .collect();
    let ok = results.iter().filter(|result| result.is_ok()).count();
    let err = results.iter().filter(|result| result.is_err()).count();
    writeln!(
        main_log.lock(),
        "ALL DONE: {} successful, {} failed",
        ok,
        err
    )?;

    Ok(())
}
