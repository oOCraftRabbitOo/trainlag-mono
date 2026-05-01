use crate::commands;

#[derive(Debug)]
pub enum Error {
    Disconnect(String),
    InvalidSignal(String),
    Connection(std::io::Error),
    Truinlag(commands::Error),
    InternalComms(&'static str),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Disconnect(err) => write!(f, "disconnected from the Truinlag engine: {}", err),
            Error::InvalidSignal(text) => {
                write!(f, "received invalid signal from engine: {}", text)
            }
            Error::Connection(err) => write!(f, "couldn't connect: {}", err),
            Error::Truinlag(err) => write!(f, "truinlag returned an error: {}", err),
            Error::InternalComms(context) => write!(
                f,
                "error sending or receiving on internal channel: {}",
                context
            ),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Connection(error)
    }
}

impl From<commands::Error> for Error {
    fn from(error: commands::Error) -> Self {
        Error::Truinlag(error)
    }
}

impl std::error::Error for Error {}
