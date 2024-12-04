#[derive(Debug)]
pub enum Error {
    FailedScan,
    WrongArgumentsCount,
    InvalidWirelessMode,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedScan => write!(f, "ssid scan failed"),
            Self::WrongArgumentsCount => write!(f, "wrong argument count provided"),
            Self::InvalidWirelessMode => write!(f, "invalid wireless mode selected"),
        }
    }
}
