use anyhow::{Context as _, Result};
use clap::Parser;
use fs_err::File;
#[cfg(not(target_family = "wasm"))]
use oparl_validator_rs::batch::validate_all;
use oparl_validator_rs::cli::{problem_report, write_detail_report, ReporterCli};
use oparl_validator_rs::client::{FileCache, NoCache, OparlClient};
use oparl_validator_rs::endpoints::get_endpoints;
use oparl_validator_rs::reporter::Reporter;
use oparl_validator_rs::storage::Storage;
use oparl_validator_rs::validate_oparl_api;
use std::io;
use std::io::BufWriter;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    name_or_url: String,
    #[arg(long)]
    cache: Option<PathBuf>,
    /// Write a report with both summary and every single case to this file
    #[arg(long, default_value = "report.txt")]
    report: PathBuf,
    /// Write progress to this file
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    #[arg(long, default_value = "log.txt")]
    log: PathBuf,
    /// Suppress output to the console
    #[arg(long)]
    quiet: bool,
}

#[cfg_attr(target_family = "wasm", allow(dead_code))]
async fn main_cli() -> Result<()> {
    let options: Options = Options::parse();

    #[cfg(not(target_family = "wasm"))]
    if options.name_or_url == "all" {
        validate_all(&options.cache, &options.report, &options.log, options.quiet).await?;
        return Ok(());
    }

    let endpoint_url = if options.name_or_url.starts_with("http") {
        options.name_or_url
    } else {
        let endpoints = get_endpoints(&NoCache).await?;

        let endpoint = endpoints
            .iter()
            .find(|x| x.title == options.name_or_url)
            .context(format!(
                "No endpoint with name '{}' found",
                options.name_or_url
            ))?;

        endpoint.url.clone()
    };

    let reporter = ReporterCli::new(Storage::new(), options.quiet, None); // TODO: Log file
    if let Some(cache_dir) = options.cache.clone() {
        validate_oparl_api::<_, _>(
            &endpoint_url,
            &reporter,
            &OparlClient::new(FileCache::new(
                cache_dir,
                endpoint_url.trim_end_matches("system").to_string(),
            )),
        )
        .await?;
    } else {
        validate_oparl_api::<_, _>(&endpoint_url, &reporter, &OparlClient::new(NoCache)).await?;
    }

    let mut report = BufWriter::new(File::create(options.report)?);
    problem_report(
        reporter.get_storage(),
        &mut io::stdout().lock(),
        &mut report,
        true,
    )?;
    write_detail_report(
        reporter.get_storage(),
        &mut io::stdout().lock(),
        &mut report,
    )?;

    Ok(())
}

#[cfg(not(target_family = "wasm"))]
#[tokio::main]
async fn main() {
    if let Err(e) = main_cli().await {
        eprintln!("💥 The validator failed: This is a bug");
        for cause in e.chain().collect::<Vec<_>>().iter() {
            eprintln!("  Caused by: {}", cause);
        }
        std::process::exit(1);
    }
}

#[cfg(target_family = "wasm")]
fn main() -> Result<()> {
    Ok(())
}
