#[derive(Deserialize)]
pub struct TeamCityResponse {
    pub status: TeamCityBuildStatus
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TeamCityBuildStatus {
    Success,
    Failure,
    Error
}