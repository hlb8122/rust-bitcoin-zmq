# Bitcoin ZMQ Wrapper

## Requirements

```bash
sudo apt install pkg-config libzmq3-dev
```

## Usage

```rust
use bitcoin_zmq::Subscriber;
use futures::{Future, Stream};

fn main() {
    let subscriber = Subscriber::new("tcp://35.222.194.216:28332", 1024);
    let print_hashes = subscriber.subscribe_raw_tx().for_each(|hash| {
        println!("raw tx: {:?}", hash);
        Ok(())
    });

    let connect = subscriber
        .connect()
        .map_err(|err| println!("err = {:?}", err));

    let runner = connect.and_then(|_| print_hashes.map_err(|err| println!("err = {:?}", err)));

    tokio::run(runner)
}
```
