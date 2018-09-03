use RemoteStatus;

pub trait RemoteIntegration {
    fn get_status(&mut self) -> RemoteStatus;
    fn get_red_id(&self) -> u16;
    fn get_green_id(&self) -> u16;
    fn get_blue_id(&self) -> u16;
}
