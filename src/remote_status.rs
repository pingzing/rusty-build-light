#[derive(Copy, Clone)]
pub enum RemoteStatus {
    Unknown, // Glowing Purple
    InProgress, // Rapid glowing green
    Passing, // Green
    Failing // Blinking red
}