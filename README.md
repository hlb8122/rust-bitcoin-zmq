# Bitcoin ZMQ Wrapper

This library provides a relatively thin wrapper around the Bitcoin ZMQ, allowing you to receive from an asynchronous stream of transaction or block data.

## Requirements

```bash
sudo apt install pkg-config libzmq3-dev
```

## Usage

```rust
use bitcoin_zmq::{SubFactory, Topic};
use futures::{lazy, Future, Stream};

fn main() {
    // Construct subscriber and broker
    let (subscriber, broker) = SubFactory::new("tcp://127.0.0.1:28332", 1024);
    let broker = broker.map_err(|err| println!("err = {:?}", err));

    // Do something with stream of raw txs
    let print_txs = subscriber.subscribe(Topic::RawTx).for_each(|raw_tx| {
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
