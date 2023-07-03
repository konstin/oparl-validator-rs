use anyhow::Result;
use fs_err::File;
use oparl_validator_rs::cli::{problem_report, write_detail_report, ReporterCli};
use oparl_validator_rs::client::{Cache, OparlClient};
use oparl_validator_rs::reporter::Reporter;
use oparl_validator_rs::storage::Storage;
use oparl_validator_rs::validate_oparl_api;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use tar::Archive;
use zstd::Decoder;

/// Normal file cache, excepts that it errors when a file is missing
struct TestCache {
    files: HashMap<String, Vec<u8>>,
    prefix: String,
}

fn normalize_archive_paths(
    entries: Result<(String, Vec<u8>)>,
) -> Option<Result<(String, Vec<u8>)>> {
    entries
        .map(|(path, content)| {
            // missing ids is saved through set_inner
            if path.ends_with("missing_ids.json") {
                return Some((path, content));
            }
            let path = path.strip_suffix(".json")?.to_string();
            Some((path, content))
        })
        .transpose()
}

impl TestCache {
    pub fn new(cache_archive: &Path, prefix: String) -> Result<Self> {
        // gzip takes ~1s, while both uncompressed and zstd take 40ms, while zstd is smaller than gzip
        let files = Archive::new(Decoder::new(File::open(cache_archive)?)?)
            .entries()?
            .filter_map(|e| e.ok())
            .map(|mut entry| -> Result<(String, Vec<u8>)> {
                // Read name and contents
                let path = entry.path()?.to_string_lossy().to_string();
                let mut content = Vec::new();
                entry.read_to_end(&mut content)?;
                Ok((path, content))
            })
            .filter_map(normalize_archive_paths)
            .collect::<Result<_>>()?;
        Ok(Self { files, prefix })
    }
}

impl Cache for TestCache {
    fn get_inner(&self, key: &str) -> Result<Option<Vec<u8>>> {
        match self.files.get(key) {
            Some(data) => Ok(Some(data.clone())),
            None => panic!("No hit for cache key: {}", key),
        }
    }

    fn set_inner(&self, key: &str, _data: &[u8]) -> Result<()> {
        panic!("Must not write in tests: {}", key)
    }

    fn get(&self, url: &str) -> Result<Option<Vec<u8>>> {
        self.get_inner(url.strip_prefix(&self.prefix).unwrap())
    }

    fn set(&self, url: &str, data: &[u8]) -> Result<()> {
        self.set_inner(url, data)
    }
}

const SUMMARY: &str = r"=== Validierungsreport: Zusammenfassung ===
Das Objekt vom Typ Meeting wurden von einem anderen Objekt verlinkt, ist aber nicht abrufbar (12 Fälle). Beispiel: https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1/organization/1-11/meeting verlinkt von https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1/organization/1-11 und 0 anderen
Das Objekt von Typ Location wurde von einem anderen Objekt verlinkt, fehlt aber in den externen Listen (35 Fälle). Beispiel: https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1/location/1-11 verlinkt von https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1/person/15 und 0 anderen
Das zwingend vorgeschriebene Feld Body.legislativeTerm fehlt (1 Fälle). Beispiel: https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1 innerhalb von https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body
Das zwingend vorgeschriebene Feld Meeting.type fehlt (21 Fälle). Beispiel: https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1/meeting/1082 innerhalb von https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1/meeting?page=11
Das zwingend vorgeschriebene Feld Paper.type fehlt (18 Fälle). Beispiel: https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1/paper/3419 innerhalb von https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1/paper?page=73
Das Feld AgendaItem.number ist angegeben, hat aber keinen Inhalt (1 Fälle). Beispiel: https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1/agendaitem/6373 innerhalb von https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/body/1/meeting?page=7
";

const LOOK_AT_REPORT: &str = "\nAlle einzelnen Fälle finden sich in report.txt\n";

#[tokio::test]
async fn test_huertgenwald() -> Result<()> {
    let reporter = ReporterCli::new(Storage::new(), false, None);
    let endpoint_url = "https://sdnetrim.kdvz-frechen.de/rim4220/webservice/oparl/v1.0/system";
    validate_oparl_api::<_, _>(
        endpoint_url,
        &reporter,
        &OparlClient::new(TestCache::new(
            Path::new("test-data/cache_huertgenwald.tar.zst"),
            endpoint_url.trim_end_matches("system").to_string(),
        )?),
    )
    .await?;

    let mut out: Vec<u8> = Vec::new();
    let mut report: Vec<u8> = Vec::new();
    problem_report(reporter.get_storage(), &mut out, &mut report, true)?;
    write_detail_report(reporter.get_storage(), &mut out, &mut report)?;

    let out = String::from_utf8(out)?;
    let report = String::from_utf8(report)?;
    assert_eq!(out, SUMMARY.to_string() + LOOK_AT_REPORT);
    assert!(
        report.starts_with(SUMMARY),
        "\n|||\n{}\n|||\n{}\n|||\n",
        &report[0..2000],
        &SUMMARY
    );
    Ok(())
}
