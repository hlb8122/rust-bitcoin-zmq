# Rust Bitcoin ZMQ

[![Build Status](https://travis-ci.org/hlb8122/rust-bitcoin-zmq.svg?branch=master)](https://travis-ci.org/hlb8122/rust-bitcoin-zmq)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Cargo](https://img.shields.io/crates/v/bitcoin-zmq.svg)](https://crates.io/crates/bitcoin-zmq)
[![Documentation](https://docs.rs/bitcoin-zmq/badge.svg)](
https://docs.rs/bitcoin-zmq)

This crate provides a relatively thin wrapper around Bitcoin ZMQ, allowing for the construction of asynchronous streams of transaction or block data.

## Requirements

```bash
sudo apt install pkg-config libzmq3-dev
```

## Usage

```rust
use bitcoin_zmq::ZMQListener;
use futures::prelude::*;

#[tokio::main]
async fn main() {
    // Construct ZMQ listenr
    let listener = ZMQListener::bind("tcp://127.0.0.1:28332")
        .await
        .expect("could not connect");

    // Do something with stream of messages
    listener
        .stream()
        .take(10)
        .try_for_each(move |raw| {
            println!("raw message: {:?}", hex::encode(raw));
            future::ok(())
        })
        .await
        .expect("zmq error'd during stream");
}
```
