use configuration::Settings;
use futures_cpupool::Builder;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use std::error::Error;
use tokio::executor::current_thread::CurrentThread;
use tokio::prelude::Stream;

// https://github.com/fede1024/rust-rdkafka/blob/master/examples/asynchronous_processing.rs
pub fn run_async_handler(config: Settings) -> Result<(), Box<Error>> {
    let mut io_loop = CurrentThread::new();

    let cpu_pool = Builder::new().pool_size(4).create();

    let mut consumer = ClientConfig::new();

    if let (Some(user), Some(pass)) = (config.username, config.password) {
        consumer
            .set("security.protocol", "SASL_SSL")
            .set("sasl.mechanisms", "PLAIN")
            .set("sasl.username", &user)
            .set("sasl.password", &pass);
    }

    let consumer = consumer
        .set("group.id", &config.consumer_group)
        .set("bootstrap.servers", &config.broker)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create::<StreamConsumer<_>>()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[&config.topic])
        .expect("Can't subscribe to specified topic");

    let handle = io_loop.handle();

    let processed_stream = consumer
        .start()
        .filter_map(|result| match result {
            Ok(msg) => Some(msg),
            Err(kafka_error) => {
                warn!("Error while receiving from Kafka: {:?}", kafka_error);
                None
            }
        }).for_each(move |msg| {
            info!("Enqueuing message for computation");
            let owned_message = msg.detach();

            Ok(())
        });

    info!("Starting event loop");
    io_loop.block_on(processed_stream).unwrap();
    info!("Stream processing terminated");
    Ok(())
}
