#[derive(Deserialize)]
pub struct JenkinsJobResponse {
    pub jobs: Vec<JenkinsJob>
}

#[derive(Deserialize)]
pub struct JenkinsJob {
    pub name: String,
    pub url: String,
    pub color: String
}

#[derive(Deserialize)]
pub struct JenkinsBuildResult {
    pub building: bool,

    #[serde(rename = "fullDisplayName")]
    pub full_display_name: String,

    #[serde(rename = "result")]
    pub build_result: JenkinsBuildStatus
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JenkinsBuildStatus {
    Success,
    Failure,
    NotBuilt,
    Aborted,
    Unstable        
}