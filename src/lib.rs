pub mod errors;

use std::sync::Arc;

use bitcoin::consensus::encode::Decodable;
use bitcoin::{util::psbt::serialize::Deserialize, Block, Transaction};
use futures::{Future, Sink, Stream};
use futures_zmq::{prelude::*, Sub};
use multiqueue::BroadcastFutReceiver;
use zmq::Context;

use errors::*;

pub struct Subscriber {
    connect: Box<Future<Item = (), Error = SubscriptionError> + Send>,
    outgoing: BroadcastFutReceiver<(Topic, Vec<u8>)>,
}

impl Subscriber {
    pub fn new(addr: &str, buffer: u64) -> Self {
        let context = Arc::new(Context::new());
        let socket = Sub::builder(context).connect(addr).filter(b"").build();

        let (incoming, outgoing) = multiqueue::broadcast_fut_queue(buffer);
        let connect = socket
            .map_err(SubscriptionError::from)
            .and_then(move |sub| {
                let classify_stream =
                    sub.stream()
                        .map_err(SubscriptionError::from)
                        .and_then(move |mut multipart| {
                            let topic = multipart.pop_front().ok_or(BitcoinError::IncompleteMsg)?;
                            let payload =
                                multipart.pop_front().ok_or(BitcoinError::IncompleteMsg)?;
                            let classification = match &*topic {
                                b"rawtx" => Topic::RawTx,
                                b"hashtx" => Topic::HashTx,
                                b"rawblock" => Topic::RawBlock,
                                b"hashblock" => Topic::HashBlock,
                                _ => return Err(BitcoinError::IncompleteMsg.into()),
                            };
                            Ok((classification, payload.to_vec()))
                        });
                incoming
                    .sink_map_err(|_| SubscriptionError::SendError)
                    .send_all(classify_stream).and_then(|_| Ok(()))
            });

        let connect = Box::new(connect);
        Subscriber { connect, outgoing }
    }

    pub fn connect(self) -> impl Future<Item = (), Error = SubscriptionError> {
        self.connect
    }

    pub fn subscribe_raw_tx(&self) -> impl Stream<Item = Vec<u8>, Error = ()> {
        self.outgoing
            .add_stream()
            .filter_map(move |(topic, payload)| {
                if topic == Topic::RawTx {
                    Some(payload)
                } else {
                    None
                }
            })
    }

    pub fn subscribe_hash_tx(&self) -> impl Stream<Item = Vec<u8>, Error = ()> {
        self.outgoing
            .add_stream()
            .filter_map(move |(topic, payload)| {
                if topic == Topic::HashTx {
                    Some(payload)
                } else {
                    None
                }
            })
    }

    pub fn subscribe_tx(&self) -> impl Stream<Item = Transaction, Error = SubscriptionError> {
        self.outgoing
            .add_stream()
            .filter_map(move |(topic, payload)| {
                if topic == Topic::Tx {
                    Some(payload)
                } else {
                    None
                }
            })
            .map_err(|_| SubscriptionError::SendError)
            .and_then(move |raw_tx| Transaction::deserialize(&raw_tx).map_err(|err| err.into()))
    }

    pub fn subscribe_raw_block(&self) -> impl Stream<Item = Vec<u8>, Error = ()> {
        self.outgoing
            .add_stream()
            .filter_map(move |(topic, payload)| {
                if topic == Topic::RawBlock {
                    Some(payload)
                } else {
                    None
                }
            })
    }

    pub fn subscribe_hash_block(&self) -> impl Stream<Item = Vec<u8>, Error = ()> {
        self.outgoing
            .add_stream()
            .filter_map(move |(topic, payload)| {
                if topic == Topic::HashBlock {
                    Some(payload)
                } else {
                    None
                }
            })
    }

    pub fn subscribe_block(&self) -> impl Stream<Item = Block, Error = SubscriptionError> {
        self.outgoing
            .add_stream()
            .filter_map(move |(topic, payload)| {
                if topic == Topic::Block {
                    Some(payload)
                } else {
                    None
                }
            })
            .map_err(|_| SubscriptionError::SendError)
            .and_then(move |raw_tx| {
                Block::consensus_decode(&mut &raw_tx[..]).map_err(|err| err.into())
            })
    }
}

#[derive(Clone, PartialEq)]
pub enum Topic {
    RawTx,
    HashTx,
    Tx,
    RawBlock,
    HashBlock,
    Block,
}
