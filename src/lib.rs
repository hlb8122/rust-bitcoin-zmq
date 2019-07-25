pub mod errors;

use std::sync::Arc;

use bus_queue::async_::*;
use futures::{future, Future, Sink, Stream};
use futures_zmq::{prelude::*, Sub};
use zmq::Context;

use errors::*;

#[derive(Clone, PartialEq)]
pub enum Topic {
    RawTx,
    HashTx,
    RawBlock,
    HashBlock,
}

pub struct SubFactory(Subscriber<(Topic, Vec<u8>)>);

impl SubFactory {
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
        let (broadcast_incoming, broadcast) = channel(capacity);

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
                            Ok((topic, payload[..].to_vec()))
                        });

                // Forward messages to broadcast channel
                broadcast_incoming
                    .sink_map_err(SubscriptionError::Channel)
                    .send_all(classify_stream)
                    .and_then(|_| Ok(()))
            });

        (SubFactory(broadcast), broker)
    }

    #[inline]
    pub fn single_stream<S>(
        addr: &str,
        topic: Topic,
    ) -> impl Future<
        Item = impl Stream<Item = Vec<u8>, Error = SubscriptionError> + Send,
        Error = SubscriptionError,
    > + Send {
        // Setup socket
        let context = Arc::new(Context::new());
        let socket = Sub::builder(context).connect(addr).filter(b"").build();

        // Connection future
        socket
            .map_err(SubscriptionError::from)
            .and_then(move |sub| {
                future::ok(
                    sub.stream()
                        .map_err(SubscriptionError::from)
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
                            // Extract payload
                            let payload =
                                multipart.pop_front().ok_or(BitcoinError::MissingPayload)?;
                            Ok(payload[..].to_vec())
                        }),
                )
            })
    }

    #[inline]
    pub fn subscribe(&self, filter_topic: Topic) -> impl Stream<Item = Vec<u8>, Error = ()> {
        self.0.clone().filter_map(move |arc_tuple| {
            if arc_tuple.0 == filter_topic {
                Some(arc_tuple.1.clone())
            } else {
                None
            }
        })
    }
}
