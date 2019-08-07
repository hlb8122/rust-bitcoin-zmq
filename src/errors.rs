use futures::sync::mpsc as fmpsc;
use std::sync::mpsc;

use bytes::Bytes;
use futures_zmq::Error as ZMQError;

use super::Topic;

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

/// Primary error type concerning the ZMQ subscription
#[derive(Debug)]
pub enum SubscriptionError {
    /// Error originating from bitcoind
    Bitcoin(BitcoinError),
    /// Error sending over the broadcast channel
    BroadcastChannel(mpsc::SendError<(Topic, Bytes)>),
    /// Error sending over single stream channel
    Channel(fmpsc::SendError<Vec<u8>>),
    /// Error in the connection to bitcoind
    Connection(ZMQError),
}

impl From<BitcoinError> for SubscriptionError {
    fn from(err: BitcoinError) -> SubscriptionError {
        SubscriptionError::Bitcoin(err)
    }
}

impl From<mpsc::SendError<(Topic, Bytes)>> for SubscriptionError {
    fn from(err: mpsc::SendError<(Topic, Bytes)>) -> SubscriptionError {
        SubscriptionError::BroadcastChannel(err)
    }
}

impl From<fmpsc::SendError<Vec<u8>>> for SubscriptionError {
    fn from(err: fmpsc::SendError<Vec<u8>>) -> SubscriptionError {
        SubscriptionError::Channel(err)
    }
}

impl From<ZMQError> for SubscriptionError {
    fn from(err: ZMQError) -> SubscriptionError {
        SubscriptionError::Connection(err)
    }
}
