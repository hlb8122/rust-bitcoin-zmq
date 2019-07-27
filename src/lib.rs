pub mod errors;

use std::sync::Arc;

use bus_queue::async_::*;
pub use bytes::Bytes;
use futures::{future, sync::mpsc, Future, Sink, Stream};
use futures_zmq::{prelude::*, Sub};
use zmq::Context;

pub use errors::*;

#[derive(Clone, PartialEq)]
pub enum Topic {
    RawTx,
    HashTx,
    RawBlock,
    HashBlock,
}

#[derive(Clone)]
pub struct ZMQSubscriber(Subscriber<(Topic, Bytes)>);

impl ZMQSubscriber {
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

    #[inline]
    pub fn single_stream(
        addr: &str,
        topic: Topic,
        capacity: usize,
    ) -> (
        Box<Stream<Item = Vec<u8>, Error = ()> + Send>,
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

    /// Subscribe
    #[inline]
    pub fn subscribe(
        self,
        filter_topic: Topic,
    ) -> impl Stream<Item = Bytes, Error = ()> + Send + Sized {
        self.0.filter_map(move |arc_tuple| {
            if arc_tuple.0 == filter_topic {
                Some(arc_tuple.1.clone())
            } else {
                None
            }
        })
    }
}
