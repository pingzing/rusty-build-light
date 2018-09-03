use failure::Error;
use integrations::jenkins_response::*;
use network::{get_basic_credentials, get_url_response};
use remote_status::RemoteStatus;
use reqwest::header::{Authorization, Headers};
use RemoteIntegration;

pub struct JenkinsIntegration {
    r: u16,
    g: u16,
    b: u16,
    username: String,
    password: String,
    base_url: String,
}

impl JenkinsIntegration {
    pub fn new(
        r: u16,
        g: u16,
        b: u16,
        username: &str,
        password: &str,
        base_url: &str,
    ) -> JenkinsIntegration {
        JenkinsIntegration {
            r: r,
            g: g,
            b: b,
            username: username.to_string(),
            password: password.to_string(),
            base_url: base_url.to_string(),
        }
    }

    fn get_status_internal(&self) -> Result<Vec<Result<JenkinsBuildStatus, Error>>, Error> {
        let url_string = format!("{base}/api/json", base = self.base_url);
        let mut auth_headers = Headers::new();
        auth_headers.set(Authorization(get_basic_credentials(
            self.username.as_str(),
            Some(self.password.clone()),
        )));

        let all_jobs_response: Result<(JenkinsJobResponse, Headers), Error> =
            get_url_response(&url_string, auth_headers.clone());

        match all_jobs_response {
            Ok((result, _)) => {
                let results = result
                    .jobs
                    .iter()
                    .filter(|job| {
                        job.color != JenkinsJobColor::Disabled
                            && job.color != JenkinsJobColor::DisabledAnime
                    })
                    .map(|job| {
                        let job_url_string = format!(
                            "{base}/job/{job}/lastBuild/api/json",
                            base = self.base_url,
                            job = job.name
                        );
                        let job_response: Result<
                            (JenkinsBuildResult, Headers),
                            Error,
                        > = get_url_response(&job_url_string, auth_headers.clone());

                        match job_response {
                            Ok((job_result, _)) => {
                                if job_result.building {
                                    Ok(JenkinsBuildStatus::Building)
                                } else {
                                    let unwrapped_result = job_result.build_result.unwrap();
                                    Ok(unwrapped_result)
                                }
                            }
                            Err(job_err) => {
                                warn!("--Jenkins--: HTTP failure when attempting to get job result for job: {}. Error: {}", &job_url_string, job_err);
                                Err(job_err)
                            }
                        }
                    })
                    .collect();
                Ok(results)
            }
            Err(err) => Err(err),
        }
    }
}

impl RemoteIntegration for JenkinsIntegration {
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
        match self.get_status_internal() {
            Ok(results) => {
                let (retrieved, not_retrieved): (
                    Vec<Result<JenkinsBuildStatus, Error>>,
                    Vec<Result<JenkinsBuildStatus, Error>>,
                ) = results.into_iter().partition(|x| x.is_ok());

                let retrieved: Vec<JenkinsBuildStatus> =
                    retrieved.into_iter().map(|x| x.unwrap()).collect();

                let retrieved_count = retrieved.len();
                let not_retrieved_count = not_retrieved.len();
                let build_failures = *(&retrieved
                    .iter()
                    .filter(|x| {
                        **x == JenkinsBuildStatus::Failure || **x == JenkinsBuildStatus::Unstable
                    })
                    .count());
                let indeterminate_count = *(&retrieved
                    .iter()
                    .filter(|x| {
                        **x != JenkinsBuildStatus::Failure
                            && **x != JenkinsBuildStatus::Unstable
                            && **x != JenkinsBuildStatus::Success
                    })
                    .count()) + not_retrieved_count;
                let build_successes = *(&retrieved
                    .iter()
                    .filter(|x| **x == JenkinsBuildStatus::Success)
                    .count());

                let builds_in_progress = *(&retrieved
                    .iter()
                    .filter(|x| **x == JenkinsBuildStatus::Building)
                    .count());

                let return_status: RemoteStatus;

                // Failure states: NONE of the builds succeeded.
                if build_successes <= 0 {
                    return RemoteStatus::Failing;
                }
                // Success, or partial success states: at least SOME builds succeeded.
                else {
                    if build_failures == 0 {
                        // If no failures, immediately report any builds-in-progress
                        if builds_in_progress > 0 {
                            return RemoteStatus::InProgress;
                        }
                        // No failures, and more successes than indeterminates
                        if build_successes > indeterminate_count {
                            return RemoteStatus::Passing;
                        }
                        // No failures, but more indeterminates than successes.
                        else {
                            return RemoteStatus::Failing;
                        }
                    // Some failures, but more successes than failures
                    } else if build_successes > build_failures {
                        return_status = RemoteStatus::Failing;
                    // Many failures, more than successes.
                    } else {
                        return_status = RemoteStatus::Failing;
                    }
                }

                info!("--Jenkins--: Retrieved {} jobs, failed to retrieve {} jobs. Of those, {} succeeded, {} failed, and {} were indeterminate.", retrieved_count, not_retrieved_count, build_successes, build_failures, indeterminate_count);
                return return_status;
            }
            Err(e) => {
                warn!(
                    "--Jenkins--: Failed to retrieve any jobs from Jenkins. Details: {}",
                    e
                );
                return RemoteStatus::Unknown;
            }
        }
    }
}
