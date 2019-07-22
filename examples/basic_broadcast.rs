use bitcoin_zmq::{SubFactory, Topic};
use futures::{lazy, Future, Stream};

fn main() {
    // Construct subscriber and broker
    let (subscriber, root_broker) = SubFactory::new("tcp://127.0.0.1:28332", 1024);
    let root_broker = root_broker.map_err(|err| println!("err = {:?}", err));

    // Grab broadcast and broker
    let (broadcast, broadcast_broker) = subscriber.broadcast(Topic::HashTx, 1024);

    // Do something with stream A
    let even_hashes = broadcast.clone().for_each(|hash| {
        println!("found even hash: {:?}", hex::encode(&hash[..]));
        Ok(())
    });

    // Do something with stream B
    let odd_hashes = broadcast.clone().for_each(|hash| {
        println!("found odd hash: {:?}", hex::encode(&hash[..]));
        Ok(())
    });

    // Pass futures to Tokio's executor
    tokio::run(lazy(|| {
        tokio::spawn(broadcast_broker);
        tokio::spawn(root_broker);
        tokio::spawn(odd_hashes);
        tokio::spawn(even_hashes);
        Ok(())
    }))
}
