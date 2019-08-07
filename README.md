# Bitcoin ZMQ Rust

A library providing a relatively thin wrapper around Bitcoin ZMQ, allowing the construction of asynchronous streams of transaction or block data.

## Requirements

```bash
sudo apt install pkg-config libzmq3-dev
```

## Usage

```rust
use bitcoin_zmq::{Topic, ZMQSubscriber};
use futures::{lazy, Future, Stream};

fn main() {
    // Construct subscription factory and broker
    let (factory, broker) = ZMQSubscriber::new("tcp://127.0.0.1:28332", 1024);
    let broker = broker.map_err(|err| println!("err = {:?}", err));

    // Do something with stream of raw txs
    let print_txs = factory.subscribe(Topic::RawTx).for_each(|raw_tx| {
        println!("raw tx: {:?}", hex::encode(raw_tx));
        Ok(())
    });

    // Pass futures to Tokio's executor
    tokio::run(lazy(|| {
        tokio::spawn(broker);
        tokio::spawn(print_txs);
        Ok(())
    }))
}

```
