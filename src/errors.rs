use futures_zmq::Error as ZMQError;

#[derive(Debug)]
pub enum SubscriptionError {
    ZMQ(ZMQError),
    SendError,
    IncompleteMsg,
    NoConnection,
}

impl From<ZMQError> for SubscriptionError {
    fn from(err: ZMQError) -> SubscriptionError {
        SubscriptionError::ZMQ(err)
    }
}
