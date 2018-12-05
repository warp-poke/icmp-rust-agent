use configuration::Settings;
use futures::lazy;
use futures::Future;
use futures_cpupool::Builder;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::Message;
use std::error::Error;
use tokio::executor::current_thread::CurrentThread;
use tokio::prelude::Stream;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RequestBenchEvent {
    domain_name: String,
    // url: String,
    warp10_endpoint: String,
    token: String,
}

// https://github.com/fede1024/rust-rdkafka/blob/master/examples/asynchronous_processing.rs
pub fn run_async_handler(config: &Settings) -> Result<(), Box<Error>> {
    let mut io_loop = CurrentThread::new();

    let cpu_pool = Builder::new().pool_size(4).create();

    let mut consumer = ClientConfig::new();

    if let (Some(user), Some(pass)) = (&config.username, &config.password) {
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

            let h = config.host.clone();
            let z = config.zone.clone();

            let process_message = cpu_pool
                .spawn(lazy(move || {
                    if let Some(payload) = owned_message.payload() {
                        check_and_post(payload, &h, &z)
                    } else {
                        Err(String::from("no payload"))
                    }
                })).or_else(|err| {
                    warn!("Error while processing message: {:?}", err);
                    Ok(())
                });
            handle.spawn(process_message);
            Ok(())
        });

    info!("Starting event loop");
    io_loop.block_on(processed_stream).unwrap();
    info!("Stream processing terminated");
    Ok(())
}

fn check_and_post(payload: &[u8], host: &str, zone: &str) -> Result<(), String> {
    let r: serde_json::Result<RequestBenchEvent> = serde_json::from_slice(payload);

    // TODO: tokio-ping
    // TODO: push to Warp10
    Ok(())
}
