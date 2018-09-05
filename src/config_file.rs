#[derive(Deserialize)]
pub struct Config {
    pub allowed_failures: u32,

    pub jenkins_username: String,
    pub jenkins_password: String,
    pub jenkins_base_url: String,
    pub jenkins_led_pins: Vec<u16>,

    pub unity_cloud_api_token: String,
    pub unity_base_url: String,
    pub unity_led_pins: Vec<u16>,
}
