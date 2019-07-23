use bitcoin_zmq::{SubFactory, Topic};
use futures::{lazy, Future, Stream};

fn main() {
    // Construct subscription factory and broker
    let (factory, broker) = SubFactory::new("tcp://127.0.0.1:28332", 1024);
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
