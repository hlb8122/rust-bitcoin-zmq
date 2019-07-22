pub mod errors;

use std::sync::Arc;

use bus_queue::async_::*;
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

pub struct SubFactory(Subscriber<(Topic, Vec<u8>)>);

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
                            Ok((topic, payload.to_vec()))
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
    pub fn subscribe(&self, filter_topic: Topic) -> impl Stream<Item = Vec<u8>, Error = ()> {
        self.0.clone().filter_map(move |ref arc_tuple| {
            if arc_tuple.0 == filter_topic {
                Some(arc_tuple.1.to_vec())
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn subscribe_filtered<F, B>(
        &self,
        filter_topic: Topic,
        filter_map: F,
    ) -> impl Stream<Item = B, Error = ()>
    where
        F: FnMut(Vec<u8>) -> Option<B>,
    {
        self.subscribe(filter_topic).filter_map(filter_map)
    }

    #[inline]
    pub fn broadcast(
        &self,
        filter_topic: Topic,
        capacity: usize,
    ) -> (Subscriber<Vec<u8>>, impl Future<Item = (), Error = ()>) {
        let stream = self.subscribe(filter_topic);
        let (incoming, broadcast) = channel(capacity);
        let broker = incoming
            .sink_map_err(|_| ())
            .send_all(stream)
            .and_then(|_| Ok(()));
        (broadcast, broker)
    }

    #[inline]
    pub fn broadcast_filtered<B, F>(
        &self,
        filter_topic: Topic,
        filter_map: F,
        buffer: usize,
    ) -> (Subscriber<B>, impl Future<Item = (), Error = ()>)
    where
        B: Clone + Send,
        F: FnMut(Vec<u8>) -> Option<B>,
    {
        let stream = self.subscribe_filtered(filter_topic, filter_map);
        let (incoming, broadcast) = channel(buffer);
        let broker = incoming
            .sink_map_err(|_| ())
            .send_all(stream)
            .and_then(|_| Ok(()));
        (broadcast, broker)
    }
}

#[inline]
pub fn broadcast<B, F>(
    stream: impl Stream<Item = (Topic, Vec<u8>), Error = ()>,
    filter_map: F,
    buffer: usize,
) -> (Subscriber<B>, impl Future<Item = (), Error = ()>)
where
    B: Clone + Send,
    F: FnMut((Topic, Vec<u8>)) -> Option<B>,
{
    let stream = stream.filter_map(filter_map);
    let (incoming, broadcast) = channel(buffer);
    let broker = incoming
        .sink_map_err(|_| ())
        .send_all(stream)
        .and_then(|_| Ok(()));
    (broadcast, broker)
}
