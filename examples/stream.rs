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
