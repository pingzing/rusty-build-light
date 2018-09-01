use errors::UnityRetrievalError;
use RemoteIntegration;
use RgbLedLight;
use integrations::unity_cloud_response::*;
use failure::Error;
use reqwest::header::{Authorization, Headers, ContentType};
use network::{get_basic_credentials, get_url_response};
use std::thread;
use std::time::Duration;

const UNITY_SLEEP_DURATION: u64 = 1000 * 50;

pub struct UnityCloudIntegration {
    r: u16,
    g: u16,
    b: u16,    
    api_token: String,
    base_url: String,
}

impl UnityCloudIntegration {
    pub fn new(r: u16, g: u16, b: u16, api_token: &str, base_url: &str) -> UnityCloudIntegration {
        UnityCloudIntegration {
            r: r,
            g: g,
            b: b,
            api_token: api_token.to_string(),
            base_url: base_url.to_string(),
        }
    }

    fn get_status(&self) -> Vec<Result<(UnityBuildStatus, Headers), UnityRetrievalError>> {
        let mut headers = Headers::new();
        let auth_header = get_basic_credentials(&self.api_token, None);
        headers.set(Authorization(auth_header));
        headers.set(ContentType::json());

        let ios_url = format!(
            "{base}/buildtargets/ios-development/builds?per_page=1",
            base = self.base_url
        );
        let ios_build_response = UnityCloudIntegration::get_platform_status(&headers, ios_url.as_str());

        let android_url = format!(
            "{base}/buildtargets/android-development/builds?per_page=1",
            base = self.base_url
        );
        let android_build_response = UnityCloudIntegration::get_platform_status(&headers, android_url.as_str());
        vec![ios_build_response, android_build_response]
    }

    fn get_platform_status(headers: &Headers, url: &str) -> Result<(UnityBuildStatus, Headers), UnityRetrievalError> {
        let unity_build_response: Result<(Vec<UnityBuild>, Headers), Error> = get_url_response(&url, headers.clone());
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

    fn update_led(&self, led: &mut RgbLedLight) {
        let unity_results = self.get_status();
        let (retrieved, not_retrieved): (
            Vec<Result<(UnityBuildStatus, Headers), UnityRetrievalError>>,
            Vec<Result<(UnityBuildStatus, Headers), UnityRetrievalError>>,
        ) = unity_results.into_iter().partition(|x| x.is_ok());

        let retrieved_results: Vec<(UnityBuildStatus, Headers)> =
            retrieved.into_iter().map(|x| x.unwrap()).collect();
        let not_retrieved_results: Vec<UnityRetrievalError> =
            not_retrieved.into_iter().map(|x| x.unwrap_err()).collect();

        if not_retrieved_results.len() > 0 {
            info!("--Unity--: At least one result not retrieved.");
            led.glow_led(RgbLedLight::BLUE);
        } else {
            let passing_builds = *(&retrieved_results
                .iter()
                .filter(|x| x.0 == UnityBuildStatus::Success)
                .count());
            let failing_builds = *(&retrieved_results
                .iter()
                .filter(|x| x.0 == UnityBuildStatus::Failure)
                .count());
            let other_status_builds = *(&retrieved_results
                .iter()
                .filter(|x| x.0 != UnityBuildStatus::Success && x.0 != UnityBuildStatus::Failure)
                .count());

            // More misc statuses than knowns
            if other_status_builds > passing_builds + failing_builds {
                info!("--Unity--: More otherstatuses than passing AND failing.");
                led.glow_led(RgbLedLight::BLUE);
            }
            // All passing or misc
            else if passing_builds > 0 && failing_builds == 0 {
                info!("--Unity--: All passing or misc.");
                led.set_led_rgb_values(RgbLedLight::GREEN);
            }
            // All failing or misc
            else if passing_builds == 0 && failing_builds > 0 {
                info!("--Unity--: All failing or misc.");
                led.blink_led(RgbLedLight::RED);
            }
            // Both failing and passing
            else if passing_builds > 0 && failing_builds > 0 {
                info!("--Unity--: At least one failing AND passing.");
                led.glow_led(RgbLedLight::TEAL);
            }
            // ?????
            else {
                info!("--Unity--: Unknown state.");
                led.glow_led(RgbLedLight::PURPLE);
            }

            info!(
                "--Unity--: {} passing builds, {} failing builds, {} builds with misc statuses.",
                passing_builds, failing_builds, other_status_builds
            );
        }
        
        // Extra-long sleep, to comply with Unity's rate-limiting.
        // The docs claim that they respond with a header that dynamically
        // tells us our limit, but that's a lie.
        thread::sleep(Duration::from_millis(UNITY_SLEEP_DURATION));
    }
}