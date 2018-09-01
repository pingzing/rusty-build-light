use RgbLedLight;

pub trait RemoteIntegration {
    fn update_led(&self, led: &mut RgbLedLight);
    fn get_red_id(&self) -> u16;
    fn get_green_id(&self) -> u16;
    fn get_blue_id(&self) -> u16;
}