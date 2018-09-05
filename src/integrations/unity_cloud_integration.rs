use errors::UnityRetrievalError;
use failure::Error;
use integrations::unity_cloud_response::*;
use network::{get_basic_credentials, get_url_response};
use remote_status::RemoteStatus;
use reqwest::header::{Authorization, ContentType, Headers};
use std::time::Duration;
use std::time::Instant;
use RemoteIntegration;

const UNITY_SLEEP_DURATION: u64 = 1000 * 60;

pub struct UnityCloudIntegration {
    r: u16,
    g: u16,
    b: u16,
    api_token: String,
    base_url: String,
    last_tick: Instant,
    last_status: RemoteStatus,
}

impl UnityCloudIntegration {
    pub fn new(r: u16, g: u16, b: u16, api_token: &str, base_url: &str) -> UnityCloudIntegration {
        UnityCloudIntegration {
            r: r,
            g: g,
            b: b,
            api_token: api_token.to_string(),
            base_url: base_url.to_string(),
            last_tick: Instant::now() - Duration::from_millis(UNITY_SLEEP_DURATION),
            last_status: RemoteStatus::Unknown,
        }
    }

    fn get_status_internal(&self) -> Vec<Result<(UnityBuildStatus, Headers), UnityRetrievalError>> {
        let mut headers = Headers::new();
        let auth_header = get_basic_credentials(&self.api_token, None);
        headers.set(Authorization(auth_header));
        headers.set(ContentType::json());

        let ios_url = format!(
            "{base}/ios-development/builds?per_page=1",
            base = self.base_url
        );
        let ios_build_response =
            UnityCloudIntegration::get_platform_status(&headers, ios_url.as_str());

        let android_url = format!(
            "{base}/android-development/builds?per_page=1",
            base = self.base_url
        );
        let android_build_response =
            UnityCloudIntegration::get_platform_status(&headers, android_url.as_str());
        vec![ios_build_response, android_build_response]
    }

    fn get_platform_status(
        headers: &Headers,
        url: &str,
    ) -> Result<(UnityBuildStatus, Headers), UnityRetrievalError> {
        let unity_build_response: Result<(Vec<UnityBuild>, Headers), Error> =
            get_url_response(&url, headers.clone());
        match unity_build_response {
            Ok((mut unity_http_result, response_headers)) => {
                if unity_http_result.len() != 0 {
                    Ok((unity_http_result.remove(0).build_status, response_headers))
                } else {
                    warn!(
                        "--Unity--: No builds retrieved from Unity Cloud for URL {}. Aborting...",
                        url
                    );
                    Err(UnityRetrievalError::NoBuildsReturned)
                }
            }
            Err(unity_http_err) => {
                warn!(
                    "--Unity--: Failure getting Unity Cloud build status for url: {}. Error: {}",
                    url, unity_http_err
                );
                Err(UnityRetrievalError::HttpError {
                    http_error_message: unity_http_err.to_string(),
                })
            }
        }
    }
}

impl RemoteIntegration for UnityCloudIntegration {
    fn get_red_id(&self) -> u16 {
        self.r
    }
    fn get_green_id(&self) -> u16 {
        self.g
    }
    fn get_blue_id(&self) -> u16 {
        self.b
    }

    fn get_status(&mut self) -> RemoteStatus {
        // Poll this as frequently as the rest, but only actually do any work
        // once every UNITY_SLEEP_DURATION, so we don't hit the API's
        // rate limit. It claims we can inspet the rate limit header we get
        // back to avoid that, but it doesn't work correctly.
        if Instant::now() - self.last_tick < Duration::from_millis(UNITY_SLEEP_DURATION) {
            let till_next = Duration::from_millis(UNITY_SLEEP_DURATION) - (Instant::now() - self.last_tick);
            info!("--Unity-- Sleeping for another {} seconds.", till_next.as_secs());
            return self.last_status;
        }

        let unity_results = self.get_status_internal();
        let (retrieved, not_retrieved): (
            Vec<Result<(UnityBuildStatus, Headers), UnityRetrievalError>>,
            Vec<Result<(UnityBuildStatus, Headers), UnityRetrievalError>>,
        ) = unity_results.into_iter().partition(|x| x.is_ok());

        let retrieved_results: Vec<(UnityBuildStatus, Headers)> =
            retrieved.into_iter().map(|x| x.unwrap()).collect();
        let not_retrieved_results: Vec<UnityRetrievalError> =
            not_retrieved.into_iter().map(|x| x.unwrap_err()).collect();

        let return_status: RemoteStatus;

        if not_retrieved_results.len() > 0 {
            info!("--Unity--: At least one result not retrieved.");
            return_status = RemoteStatus::Unknown;
        } else {
            let passing_builds = *(&retrieved_results
                .iter()
                .filter(|x| x.0 == UnityBuildStatus::Success)
                .count());
            let failing_builds = *(&retrieved_results
                .iter()
                .filter(|x| x.0 == UnityBuildStatus::Failure)
                .count());
            let in_progress_builds = *(&retrieved_results
                .iter()
                .filter(|x| {
                    x.0 == UnityBuildStatus::Queued
                        || x.0 == UnityBuildStatus::SentToBuilder
                        || x.0 == UnityBuildStatus::Started
                        || x.0 == UnityBuildStatus::Restarted
                })
                .count());
            let other_status_builds = *(&retrieved_results
                .iter()
                .filter(|x| {
                    x.0 != UnityBuildStatus::Success
                        && x.0 != UnityBuildStatus::Failure
                        && x.0 != UnityBuildStatus::Queued
                        && x.0 != UnityBuildStatus::SentToBuilder
                        && x.0 != UnityBuildStatus::Started
                        && x.0 != UnityBuildStatus::Restarted
                })
                .count());

            // More misc statuses than knowns
            if other_status_builds > passing_builds + failing_builds + in_progress_builds {
                info!("--Unity--: More otherstatuses than passing AND failing.");
                return_status = RemoteStatus::Unknown;
            }
            // No failures and at least one building
            else if failing_builds == 0 && in_progress_builds > 0 {
                info!("--Unity--: No failures and at least one building");
                return_status = RemoteStatus::InProgress;
            }
            // All passing or misc
            else if passing_builds > 0 && failing_builds == 0 {
                info!("--Unity--: All passing or misc.");
                return_status = RemoteStatus::Passing;
            }
            // All failing or misc
            else if passing_builds == 0 && failing_builds > 0 {
                info!("--Unity--: All failing or misc.");
                return_status = RemoteStatus::Failing;
            }
            // Both failing and passing
            else if passing_builds > 0 && failing_builds > 0 {
                info!("--Unity--: At least one failing AND passing.");
                return_status = RemoteStatus::Failing;
            }
            // ?????
            else {
                info!("--Unity--: Unknown state.");
                return_status = RemoteStatus::Unknown;
            }

            info!(
                "--Unity--: {} passing builds, {} failing builds, {} builds in progress, {} builds with misc statuses.",
                passing_builds, failing_builds, in_progress_builds, other_status_builds
            );
        }
        self.last_tick = Instant::now();
        self.last_status = return_status;
        return return_status;
    }
}
