#[derive(Deserialize)]
pub struct Config {
    pub jenkins_username: String,
    pub jenkins_password: String,
    pub jenkins_base_url: String,

    pub unity_cloud_api_token: String,
    pub unity_base_url: String,

    pub team_city_username: String,
    pub team_city_password: String,
    pub team_city_base_url: String
}