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
    pub build_result: String
}