use bitcoin_zmq::Subscriber;
use futures::{Future, Stream, lazy};

fn main() {
    // Declare subscriber
    let subscriber = Subscriber::new("tcp://127.0.0.1:28332", 1024);

    // Do something with stream
    let print_hashes = subscriber.subscribe_raw_tx().for_each(|hash| {
        println!("raw tx: {:?}", hash);
        Ok(())
    });

    // Connection future
    let connect = subscriber
        .connect()
        .map_err(|err| println!("err = {:?}", err));

    tokio::run(lazy(|| {
        tokio::spawn(connect); // Connect
        tokio::spawn(print_hashes); // Do something
        Ok(())
    }) )
}