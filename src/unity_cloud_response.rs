#[derive(Deserialize)]
pub struct UnityBuild {
    #[serde(rename = "buildStatus")]
    pub build_status: UnityBuildStatus,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum UnityBuildStatus {
    Queued,
    SentToBuilder,
    Started,
    Restarted,
    Success,
    Failure,
    Canceled,
    Unknown,
}
