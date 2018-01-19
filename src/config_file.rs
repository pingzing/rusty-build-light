#[derive(Deserialize)]
pub struct Config {
    pub jenkins_username: String,
    pub jenkins_password: String,    
    pub unity_cloud_api_token: String,
    pub team_city_username: String,
    pub team_city_password: String
}