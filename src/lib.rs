use crate::reporter::Reporter;
use crate::semaphore::Semaphore;
use crate::visit::Visitable;
use anyhow::{Context, Error, Result};
use client::{Cache, OparlClient};
use external_list::ExternalList;
use futures::prelude::stream::FuturesUnordered;
use futures::StreamExt;
use reporter::ProgressBarWrapper;
use schema::{Body, Meeting, Organization, Paper, Person, System};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
#[cfg(not(target_family = "wasm"))]
use std::time::Duration;
#[cfg(not(target_family = "wasm"))]
use tokio::time::sleep;
use visit::{OparlObject, Problem, ProblemKind};

pub mod batch;
pub mod cli;
pub mod client;
pub mod endpoints;
pub mod external_list;
pub mod reporter;
pub mod schema;
pub mod semaphore;
pub mod storage;
pub mod visit;
pub mod wasm;

/// Go through an external list with objects of type T
async fn process_list<T: DeserializeOwned + OparlObject, R: Reporter, C: Cache>(
    first_url: &str,
    reporter: &R,
    client: &OparlClient<C>,
    progress_bar: R::ProgressBar,
) -> Result<()> {
    let mut next_url = Some(first_url.to_string());
    let mut object_count = 0;
    let mut page_count = 0;

    // If we have next page url, load it, else exit
    while let Some(current_url) = next_url {
        progress_bar.set_message(current_url.to_string());
        // Very naive retry implementation
        let mut counter = 0;
        let page: Result<ExternalList<T>, _> = loop {
            match client.get(&current_url).await {
                Ok(page) => break page,
                Err(err) => {
                    counter += 1;

                    if counter < 3 {
                        reporter.add_problem(Problem {
                            detail: format!(
                                "Abfrage von {} ist fehlgeschlagen, wird erneut versucht: {}",
                                current_url, err
                            ),
                            path: "".to_string(),
                            kind: ProblemKind::HttpRequestFailedRetrying,
                        });
                        // TODO(konstin): Also wait when using wasm
                        #[cfg(not(target_family = "wasm"))]
                        {
                            sleep(Duration::from_secs(5)).await;
                        }
                        continue;
                    } else {
                        reporter.add_problem(Problem {
                            detail: format!(
                                "Die Abfrage {} ist zu oft fehlgeschlagen, eine Validierung der Liste ist nicht möglich: {}",
                                current_url, err
                            ),
                            path: "".to_string(),
                            kind: ProblemKind::HttpRequestFailedTooOften,
                        });
                        // The page failed to often, we can't do anything with that list anymore
                        return Ok(());
                    }
                }
            }
        };

        page_count += 1;

        // If the page matches the schema, take it, otherwise try to at least extract the next url
        let page = match page {
            Ok(page) => {
                next_url = page.links.next.to_owned();
                if let Some(total_pages) = page.pagination.total_pages {
                    progress_bar.set_length(total_pages as u64);
                }

                page
            }
            Err((value, error)) => {
                reporter.add_problem(Problem {
                    detail: format!(
                        "Die Liste unter {} passt nicht zum Schema von externen Listen: {}",
                        current_url, error
                    ),
                    path: String::new(),
                    kind: ProblemKind::InvalidJson,
                });
                // Try to extract $.links.next so that we can continue with the next page even
                // if this one failed
                next_url = value
                    .as_object()
                    .and_then(|x| x.get("links"))
                    .and_then(|x| x.as_object())
                    .and_then(|x| x.get("next"))
                    .and_then(|x| x.as_str())
                    .map(|x| x.to_string());
                continue;
            }
        };

        // Actual page logic
        object_count += page.data.len();
        for object in page.data {
            object.visit(reporter, &current_url, object.get_id(), &current_url);
        }
        progress_bar.inc(1);
        reporter.finish_page();
    }

    progress_bar.finish_with_message(format!(
        "Found {} {} in {} pages",
        object_count,
        T::type_name(),
        page_count
    ));
    Ok(())
}

/// Check whether the urls that were not in the list can be loaded individually
#[allow(clippy::await_holding_lock)] // For this function it doesn't matter anymore
async fn analyze_missing_urls<P: Reporter, C: Cache>(
    client: &OparlClient<C>,
    reporter: &P,
) -> Result<()> {
    let known_ids = reporter.get_storage().known_ids.lock();
    let linked_ids = reporter.get_storage().linked_ids.lock();
    let missing_urls: Vec<_> = linked_ids
        .keys()
        .map(|(_, url)| url)
        .filter(|url| !known_ids.contains(url.as_str()))
        .collect();

    let mut known_statuses: HashMap<String, bool> =
        if let Some(bytes) = client.cache.get_inner("missing_ids.json")? {
            serde_json::from_slice(&bytes).context("missing_ids.json is corrupted")?
        } else {
            HashMap::new()
        };

    /// Check whether the url is reachable, as a function due to typechecker limitations
    async fn head(semaphore: &Semaphore, url: &str, reporter: &impl Reporter) -> (String, bool) {
        let _permit = semaphore.acquire().await.unwrap();
        let status = match reqwest::Client::new().head(url).send().await {
            Ok(response) => response.status().is_success(),
            Err(err) => {
                reporter.add_message(&format!("Error in HEAD request to {}: {}", url, err));
                false
            }
        };
        (url.to_string(), status)
    }

    let semaphore = Semaphore::new(50);
    let mut futures: FuturesUnordered<_> = missing_urls
        .iter()
        .filter(|url| !known_statuses.contains_key(&url.to_string()))
        .map(|url| head(&semaphore, url, reporter)) // .boxed()
        .collect();

    if futures.is_empty() && known_statuses.is_empty() {
        // TODO: Report this on wasm
        println!("No missing objects");
        return Ok(());
    }

    if !futures.is_empty() {
        let bar = reporter.add_bar("Fehlende Objekte überprüfen");
        bar.set_length(futures.len() as u64);
        while let Some((url, status)) = futures.next().await {
            bar.inc(1);
            known_statuses.insert(url, status);
        }

        let count = linked_ids
            .iter()
            .filter(|((_type_name, url), _sources)| missing_urls.contains(&url))
            .count();

        bar.finish_with_message(format!("{} fehlende Objekte gefunden", count));

        client
            .cache
            .set_inner("missing_ids.json", &serde_json::to_vec(&known_statuses)?)?;
    }

    for ((type_name, url), sources) in linked_ids.iter() {
        if !missing_urls.contains(&url) {
            continue;
        }
        reporter.add_problem(Problem {
            detail: format!(
                "{} verlinkt von {} und {} anderen",
                url,
                sources[0],
                sources.len() - 1
            ),
            path: "".to_string(),
            kind: if *known_statuses.get(url).unwrap() {
                ProblemKind::ObjectNotInList(type_name)
            } else {
                ProblemKind::ObjectMissing(type_name)
            },
        })
    }
    Ok(())
}

/// Validates one entire oparl API
pub async fn validate_oparl_api<T: Reporter, C: Cache>(
    entrypoint: &str,
    reporter: &T,
    client: &OparlClient<C>,
) -> Result<()> {
    let initial_request = match client.get(entrypoint).await {
        Ok(ok) => ok,
        Err(err) => {
            let mut err_formatted = String::new();
            for cause in err.chain().collect::<Vec<_>>().iter() {
                err_formatted += &format!(". Caused by: {}", cause);
            }
            reporter.add_problem(Problem {
                detail:
                    format!("Der Endpunkt unter {} konnte nicht erreicht werden, damit ist keine Validierung möglich: {}", entrypoint, err_formatted),
                path: "".to_string(),
                kind: ProblemKind::HttpRequestFailedTooOften,
            });
            return Ok(());
        }
    };

    let system: System = initial_request
        .map_err(|(_, error)| Error::msg(error))
        .context("Could not parse entrypoint into System schema")?;

    let object = &system;
    let url = entrypoint;
    object.visit(reporter, url, object.get_id(), "");
    let body_id = match system.body {
        Some(body_id) => body_id,
        None => {
            reporter.add_problem(Problem {
                detail:
                    "Das System-Objekt hat kein Body-Feld, damit ist keine Validierung möglich."
                        .to_string(),
                path: "System.body".to_string(),
                kind: ProblemKind::RequiredFieldMissing,
            });
            return Ok(());
        }
    };

    let body_list: ExternalList<Body> = match client.get(&body_id).await {
        Ok(ok) => ok
            .map_err(|(_, error)| Error::msg(error))
            .context("Could not parse body list into schema")?,
        Err(err) => {
            reporter.add_problem(Problem {
                detail:
                    format!("Der Körperschaftsliste unter {} konnte nicht erreicht werden, damit ist keine Validierung möglich: {}", *body_id, err),
                path: "".to_string(),
                kind: ProblemKind::HttpRequestFailedTooOften,
            });
            return Ok(());
        }
    };

    for body in body_list.data {
        body.visit(reporter, &body_id, body.get_id(), "");

        let first_paper_url = body.paper.context("body has no papers")?;
        let first_organization_url = body.organization.context("body has no organization")?;
        let first_person_url = body.person.context("body has no person")?;
        let first_meeting_url = body.meeting.context("body has no meeting")?;

        let progress_bar_paper = reporter.add_bar("paper");
        let progress_bar_organization = reporter.add_bar("organization");
        let progress_bar_person = reporter.add_bar("person");
        let progress_bar_meeting = reporter.add_bar("meeting");

        let paper_future =
            process_list::<Paper, _, _>(&first_paper_url, reporter, client, progress_bar_paper);
        let organization_future = process_list::<Organization, _, _>(
            &first_organization_url,
            reporter,
            client,
            progress_bar_organization,
        );
        let person_future =
            process_list::<Person, _, _>(&first_person_url, reporter, client, progress_bar_person);
        let meeting_future = process_list::<Meeting, _, _>(
            &first_meeting_url,
            reporter,
            client,
            progress_bar_meeting,
        );

        futures::try_join!(
            paper_future,
            organization_future,
            person_future,
            meeting_future
        )?;
    }

    analyze_missing_urls(client, reporter).await?;

    Ok(())
}
