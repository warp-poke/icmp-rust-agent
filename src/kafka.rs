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
use trust_dns_resolver::config::*;
use trust_dns_resolver::AsyncResolver;

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

    // Spawning DNS handler
    let (resolver, background) =
        AsyncResolver::new(ResolverConfig::default(), ResolverOpts::default());
    io_loop.spawn(background);

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
                .spawn(
                    lazy(move || {
                        debug!("received a Kafka message");
                        if let Some(payload) = owned_message.payload() {
                            // deserialize kafka msg
                            let r: serde_json::Result<
                                RequestBenchEvent,
                            > = serde_json::from_slice(payload);
                            Ok(r)
                        } else {
                            Err(String::from("no payload"))
                        }
                    }).and_then(|msg| {
                        debug!("resolving DNS");
                        match msg {
                            Ok(request) => {
                                resolver.lookup_ip(request.domain_name.as_str());
                                // TODO: resolve future
                                // TODO: inject IP instead of lookup in a new Request
                                Ok(request)
                            }
                            Err(e) => Err(String::from(format!("DNS error: {}", e))),
                        }
                        // Construct a new Resolver with default configuration options
                    }).and_then(move |msg| {
                        info!("pinging");
                        // TODO
                        Ok(())
                    }),
                ).or_else(|err| {
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
