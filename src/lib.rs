//! # Bitcoin ZMQ
//!
//! A library providing a relatively thin wrapper around Bitcoin ZMQ,
//! allowing the construction of asynchronous streams of transaction
//! or block data.

pub mod errors;

use std::sync::Arc;

use bus_queue::async_::*;
pub use bytes::Bytes;
use futures::{future, sync::mpsc, Future, Sink, Stream};
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

/// Factory object allowing constuction of single stream channels and
/// broadcast channels.
///
/// Cloning to receive additional factories does not increase overhead
/// when compared to using the subscribe method.
#[derive(Clone)]
pub struct ZMQSubscriber(Subscriber<(Topic, Bytes)>);

impl ZMQSubscriber {
    /// Constructs a new factory paired with a future representing the connection/broker.
    #[inline]
    pub fn new(
        addr: &str,
        capacity: usize,
    ) -> (
        Self,
        impl Future<Item = (), Error = SubscriptionError> + Send,
    ) {
        // Setup socket
        let context = Arc::new(Context::new());
        let socket = Sub::builder(context).connect(addr).filter(b"").build();

        // Broadcast channel
        let (broadcast_incoming, subscriber) = channel(capacity);

        // Connection future
        let broker = socket
            .map_err(SubscriptionError::from)
            .and_then(move |sub| {
                let classify_stream =
                    sub.stream()
                        .map_err(SubscriptionError::from)
                        .and_then(move |mut multipart| {
                            // Parse multipart
                            let raw_topic =
                                multipart.pop_front().ok_or(BitcoinError::MissingTopic)?;
                            let payload =
                                multipart.pop_front().ok_or(BitcoinError::MissingPayload)?;
                            let topic = match &*raw_topic {
                                b"rawtx" => Topic::RawTx,
                                b"hashtx" => Topic::HashTx,
                                b"rawblock" => Topic::RawBlock,
                                b"hashblock" => Topic::HashBlock,
                                _ => return Err(BitcoinError::UnexpectedTopic.into()),
                            };
                            Ok((topic, Bytes::from(&payload[..])))
                        });

                // Forward messages to broadcast channel
                broadcast_incoming
                    .sink_map_err(SubscriptionError::BroadcastChannel)
                    .send_all(classify_stream)
                    .and_then(|_| Ok(()))
            });

        (ZMQSubscriber(subscriber), broker)
    }

    /// Construct a single stream filtered by topic.
    ///
    /// The stream is paired with a future representing the connection.
    #[inline]
    pub fn single_stream(
        addr: &str,
        topic: Topic,
        capacity: usize,
    ) -> (
        Box<dyn Stream<Item = Vec<u8>, Error = ()> + Send>,
        impl Future<Item = (), Error = SubscriptionError> + Send,
    ) {
        // Setup socket
        let context = Arc::new(Context::new());
        let socket = Sub::builder(context).connect(addr).filter(b"").build();

        // Stream channel
        let (payload_in, payload_out) = mpsc::channel(capacity);

        // Connection future
        let broker = socket
            .map_err(SubscriptionError::from)
            .and_then(move |sub| {
                future::ok(
                    sub.stream()
                        .map_err(SubscriptionError::Connection)
                        .and_then(move |mut multipart| {
                            // Extract topic
                            let raw_topic =
                                multipart.pop_front().ok_or(BitcoinError::MissingTopic)?;
                            Ok((raw_topic, multipart))
                        })
                        .filter_map(move |(raw_topic, multipart)| {
                            // Filter by topic
                            match (&*raw_topic, &topic) {
                                (b"rawtx", Topic::RawTx) => (),
                                (b"hashtx", Topic::HashTx) => (),
                                (b"rawblock", Topic::RawBlock) => (),
                                (b"hashblock", Topic::HashBlock) => (),
                                _ => return None,
                            };
                            Some(multipart)
                        })
                        .and_then(move |mut multipart| {
                            // Forward stream
                            let payload =
                                multipart.pop_front().ok_or(BitcoinError::MissingPayload)?;
                            Ok(payload[..].to_vec())
                        }),
                )
            })
            .and_then(|stream| {
                // Forward stream
                payload_in
                    .sink_map_err(|e| e.into())
                    .send_all(stream)
                    .and_then(|_| Ok(()))
            });
        (Box::new(payload_out), broker)
    }

    /// Subscribe to one-of-many streams filtered by topic.
    ///
    /// The stream is paired with a future representing the connection
    /// and the broker coordinating the broadcast channel.
    ///
    /// The return type is Bytes which prevents unecessary clones while
    /// streams are being handled by the broker.
    ///
    /// This does not consume the factory.
    #[inline]
    pub fn subscribe(
        &self,
        filter_topic: Topic,
    ) -> impl Stream<Item = Bytes, Error = ()> + Send + Sized {
        self.0.clone().filter_map(move |arc_tuple| {
            if arc_tuple.0 == filter_topic {
                Some(arc_tuple.1.clone())
            } else {
                None
            }
        })
    }
}
