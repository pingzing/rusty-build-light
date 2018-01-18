#[derive(Deserialize)]
pub struct Config {
    pub jenkins_username: String,
    pub jenkins_password: String,    
    pub unity_cloud_api_token: String
}