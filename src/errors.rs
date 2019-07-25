use std::sync::mpsc;

use bytes::Bytes;
use futures_zmq::Error as ZMQError;

use super::Topic;

#[derive(Debug)]
pub enum BitcoinError {
    MissingTopic,
    MissingPayload,
    UnexpectedTopic,
}

#[derive(Debug)]
pub enum SubscriptionError {
    Bitcoin(BitcoinError),
    Channel(mpsc::SendError<(Topic, Bytes)>),
    Connection(ZMQError),
}

impl From<BitcoinError> for SubscriptionError {
    fn from(err: BitcoinError) -> SubscriptionError {
        SubscriptionError::Bitcoin(err)
    }
}

impl From<mpsc::SendError<(Topic, Bytes)>> for SubscriptionError {
    fn from(err: mpsc::SendError<(Topic, Bytes)>) -> SubscriptionError {
        SubscriptionError::Channel(err)
    }
}

impl From<ZMQError> for SubscriptionError {
    fn from(err: ZMQError) -> SubscriptionError {
        SubscriptionError::Connection(err)
    }
}
