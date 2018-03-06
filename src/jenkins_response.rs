#[derive(Deserialize)]
pub struct JenkinsJobResponse {
    pub jobs: Vec<JenkinsJob>
}

#[derive(Deserialize)]
pub struct JenkinsJob {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize)]
pub struct JenkinsBuildResult {
    pub building: bool,

    #[serde(rename = "result")]
    pub build_result: Option<JenkinsBuildStatus>
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JenkinsBuildStatus {
    Success,
    Failure,
    NotBuilt,
    Aborted,
    Unstable,
    Building // Doesn't actually exist in Jenkins, but we do some transformation when returning it to make life simpler
}