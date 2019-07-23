pub mod errors;

use std::sync::Arc;

use bus_queue::async_::*;
use bytes::Bytes;
use futures::{Future, Sink, Stream};
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

pub struct SubFactory(Subscriber<(Topic, Bytes)>);

impl SubFactory {
    #[inline]
    pub fn new(
        addr: &str,
        capacity: usize,
    ) -> (
        Self,
        Box<Future<Item = (), Error = SubscriptionError> + Send>,
    ) {
        let context = Arc::new(Context::new());
        let socket = Sub::builder(context).connect(addr).filter(b"").build();

        // Broadcast queues
        let (broadcast_incoming, broadcast) = channel(capacity);

        // Connection future
        let connect = socket
            .map_err(SubscriptionError::from)
            .and_then(move |sub| {
                let classify_stream =
                    sub.stream()
                        .map_err(SubscriptionError::from)
                        .and_then(move |mut multipart| {
                            // Parse multipart
                            let raw_topic = multipart
                                .pop_front()
                                .ok_or(SubscriptionError::IncompleteMsg)?;
                            let payload = multipart
                                .pop_front()
                                .ok_or(SubscriptionError::IncompleteMsg)?;
                            let topic = match &*raw_topic {
                                b"rawtx" => Topic::RawTx,
                                b"hashtx" => Topic::HashTx,
                                b"rawblock" => Topic::RawBlock,
                                b"hashblock" => Topic::HashBlock,
                                _ => return Err(SubscriptionError::IncompleteMsg),
                            };
                            Ok((topic, Bytes::from(&payload[..])))
                        });

                // Forward messages to broadcast streams
                broadcast_incoming
                    .sink_map_err(|_| SubscriptionError::SendError)
                    .send_all(classify_stream)
                    .and_then(|_| Ok(()))
            });

        let broker = Box::new(connect);
        (SubFactory(broadcast), broker)
    }

    #[inline]
    pub fn subscribe(&self, filter_topic: Topic) -> impl Stream<Item = Bytes, Error = ()> {
        self.0.clone().filter_map(move |arc_tuple| {
            if arc_tuple.0 == filter_topic {
                Some(arc_tuple.1.clone())
            } else {
                None
            }
        })
    }
}
