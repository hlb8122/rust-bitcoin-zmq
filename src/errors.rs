use futures::sync::mpsc as fmpsc;
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
    BroadcastChannel(mpsc::SendError<(Topic, Bytes)>),
    Channel(fmpsc::SendError<Vec<u8>>),
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
