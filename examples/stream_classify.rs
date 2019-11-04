use bitcoin_zmq::{Topic, ZMQListener};
use futures::prelude::*;

#[tokio::main]
async fn main() {
    // Construct ZMQ listenr
    let listener = ZMQListener::bind("tcp://127.0.0.1:28332")
        .await
        .expect("could not connect");

    // Do something with specific topic messages
    listener
        .stream_classified()
        .take(10)
        .try_for_each(|(topic, raw)| {
            if topic == Topic::HashTx {
                println!("raw tx hash: {:?}", hex::encode(raw));
            }
            future::ok(())
        })
        .await
        .expect("zmq error'd during stream");
}
