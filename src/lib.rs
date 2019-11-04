//! # Bitcoin ZMQ
//!
//! A library providing a relatively thin wrapper around Bitcoin ZMQ,
//! allowing the construction of asynchronous streams of transaction
//! or block data.

pub mod errors;

use std::sync::Arc;

use futures::{
    compat::{Future01CompatExt, Stream01CompatExt},
    prelude::*,
};
pub use futures_zmq::Error as ZMQError;
use futures_zmq::{prelude::*, Sub};
use zmq::Context;

pub use errors::*;

/// Topics provided by the bitcoind ZMQ
#[derive(Clone, PartialEq)]
pub enum Topic {
    /// Raw transaction topic
    RawTx,
    /// Transaction hash topic
    HashTx,
    /// Raw block topic
    RawBlock,
    /// Block hash topic
    HashBlock,
}

/// An object representing the ZMQ stream connected to a remote endpoint.
pub struct ZMQListener {
    subscriber: Sub,
}

impl ZMQListener {
    /// Creates a new ZmqListener which will be bound to the specified address.
    ///
    /// The returned listener is ready to accept messages once the future resolves.
    pub async fn bind(addr: &str) -> Result<Self, ZMQError> {
        // Setup socket
        let context = Arc::new(Context::new());
        let sub_old = Sub::builder(context).connect(addr).filter(b"").build();
        let subscriber = sub_old.compat().await?; // Convert from futures 0.1 to 0.3
        let listener = ZMQListener { subscriber };
        Ok(listener)
    }

    /// Listen to stream of ZMQ messages from bitcoind.
    pub fn stream(self) -> impl Stream<Item = Result<Vec<u8>, SubscriptionError>> {
        let stream = self.subscriber.stream().compat().map(move |multipart_res| {
            // Parse multipart
            let mut multipart = multipart_res?;
            multipart.pop_front().ok_or(BitcoinError::MissingTopic)?;
            let payload = multipart.pop_front().ok_or(BitcoinError::MissingPayload)?;
            Ok(payload.to_vec())
        });

        Box::pin(stream)
    }

    /// Listen to stream of ZMQ messages from bitcoind.
    ///
    /// Stream items are a paired with their associated topic.
    pub fn stream_classified(
        self,
    ) -> impl Stream<Item = Result<(Topic, Vec<u8>), SubscriptionError>> {
        let stream = self.subscriber.stream().compat().map(move |multipart_res| {
            // Parse multipart
            let mut multipart = multipart_res?;
            let raw_topic = multipart.pop_front().ok_or(BitcoinError::MissingTopic)?;
            let payload = multipart.pop_front().ok_or(BitcoinError::MissingPayload)?;
            let topic = match &*raw_topic {
                b"rawtx" => Topic::RawTx,
                b"hashtx" => Topic::HashTx,
                b"rawblock" => Topic::RawBlock,
                b"hashblock" => Topic::HashBlock,
                _ => return Err(BitcoinError::UnexpectedTopic.into()),
            };
            Ok((topic, payload.to_vec()))
        });

        Box::pin(stream)
    }
}
