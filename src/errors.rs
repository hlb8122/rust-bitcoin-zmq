use bitcoin::consensus::encode::Error as EncodingError;
use futures_zmq::Error as ZMQError;

#[derive(Debug)]
pub enum SubscriptionError {
    ZMQ(ZMQError),
    Bitcoin(BitcoinError),
    SendError,
}

impl From<ZMQError> for SubscriptionError {
    fn from(err: ZMQError) -> SubscriptionError {
        SubscriptionError::ZMQ(err)
    }
}

#[derive(Debug)]
pub enum BitcoinError {
    DecodingError(EncodingError),
    IncompleteMsg,
}

impl From<EncodingError> for SubscriptionError {
    fn from(err: EncodingError) -> SubscriptionError {
        SubscriptionError::Bitcoin(BitcoinError::DecodingError(err))
    }
}

impl From<BitcoinError> for SubscriptionError {
    fn from(err: BitcoinError) -> SubscriptionError {
        SubscriptionError::Bitcoin(err)
    }
}
