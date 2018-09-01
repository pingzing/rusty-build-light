use RgbLedLight;

pub trait RemoteIntegration {
    fn update_led(&self, led: &mut RgbLedLight);
}