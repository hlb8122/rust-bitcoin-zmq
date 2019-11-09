use std::fmt;

pub use futures_zmq::Error as ZMQError;

/// Errors caused by bitcoind
#[derive(Debug)]
pub enum BitcoinError {
    /// Topic is missing
    MissingTopic,
    /// Payload is missing
    MissingPayload,
    /// Unexpected topic
    UnexpectedTopic,
}

impl fmt::Display for BitcoinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            BitcoinError::MissingTopic => "missing topic",
            BitcoinError::MissingPayload => "missing payload",
            BitcoinError::UnexpectedTopic => "unexpected topic",
        };
        write!(f, "{}", printable)
    }
}

/// Primary error type concerning the ZMQ subscription
#[derive(Debug)]
pub enum SubscriptionError {
    /// Malformed multipart supplied by bitcoind
    Bitcoin(BitcoinError),
    /// Error in the connection to bitcoind
    Connection(ZMQError),
}

impl fmt::Display for SubscriptionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SubscriptionError::Bitcoin(err) => err.fmt(f),
            SubscriptionError::Connection(err) => err.fmt(f),
        }
    }
}

impl From<BitcoinError> for SubscriptionError {
    fn from(err: BitcoinError) -> SubscriptionError {
        SubscriptionError::Bitcoin(err)
    }
}

impl From<ZMQError> for SubscriptionError {
    fn from(err: ZMQError) -> SubscriptionError {
        SubscriptionError::Connection(err)
    }
}
