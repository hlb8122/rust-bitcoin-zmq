use futures_zmq::Error as ZMQError;
use std::sync::mpsc;
use super::Topic;
// use bytes::Bytes;

#[derive(Debug)]
pub enum BitcoinError {
    MissingTopic,
    MissingPayload,
    UnexpectedTopic
}

#[derive(Debug)]
pub enum SubscriptionError {
    Bitcoin(BitcoinError),
    Channel(mpsc::SendError<(Topic, Vec<u8>)>),
    Connection(ZMQError),
}

impl From<BitcoinError> for SubscriptionError {
    fn from(err: BitcoinError) -> SubscriptionError {
        SubscriptionError::Bitcoin(err)
    }
}

impl From<mpsc::SendError<(Topic, Vec<u8>)>> for SubscriptionError {
    fn from(err: mpsc::SendError<(Topic, Vec<u8>)>) -> SubscriptionError {
        SubscriptionError::Channel(err)
    }
}

impl From<ZMQError> for SubscriptionError {
    fn from(err: ZMQError) -> SubscriptionError {
        SubscriptionError::Connection(err)
    }
}
