use crate::configuration::Settings;

use rdkafka::{
    config::ClientConfig,
    Message,
    consumer::{StreamConsumer, Consumer},
};
use tokio::prelude::*;
use tokio_async_await::await;
use trust_dns_resolver::{AsyncResolver, config::*};
use log::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RequestBenchEvent {
    domain_name: String,
    // url: String,
    warp10_endpoint: String,
    token: String,
}

pub fn run_async_handler(config: Settings) {
    tokio::run_async(async move {

        let mut consumer_builder = ClientConfig::new();

        if let (Some(user), Some(pass)) = (config.username.clone(), config.password.clone()) {
            consumer_builder
                .set("security.protocol", "SASL_SSL")
                .set("sasl.mechanisms", "PLAIN")
                .set("sasl.username", &user)
                .set("sasl.password", &pass);
    }

        let consumer: StreamConsumer = consumer_builder
            .set("group.id", &config.consumer_group.clone())
            .set("bootstrap.servers", &config.broker.clone())
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .create()
            .expect("Consumer creation failed");

        consumer
            .subscribe(&[&config.topic])
            .expect("Can't subscribe to specified topic");

        let mut messages_stream = consumer.start();
        debug!("starting stream consumer");

        // kafka ===================
        while let Some(message) = await!(messages_stream.next()) {
            debug!("got a new message");

            match message {
                Err(_) => error!("Error while reading from stream."),
                Ok(Err(e)) => error!("Kafka error: {}", e),
                Ok(Ok(m)) => {
                    let m_owned = m.detach();
                    let payload = match m_owned.payload_view::<str>() { // TODO find a better for extract the payload
                        None => "",
                        Some(Ok(s)) => s,
                        Some(Err(e)) => {
                            error!("Error while deserializing message payload: {:?}", e);
                            ""
                        },
                    }.to_string();  // transform str -> String to remove borrow issue

                    // DNS =========================
                    // TODO move this part in his own method with a async fn
                    tokio::spawn_async(async move {
                        let (resolver, background) = AsyncResolver::new(
                            ResolverConfig::default(),
                            ResolverOpts::default()
                        );

                        info!("lookup for the ip address of {}", payload);

                        tokio::spawn(background);

                        let res = await!(resolver.lookup_ip(payload.as_str()));

                        if let Ok(addresses) = res {
                            // PING =====================
                            // TODO ping all the addresses and not juste one AND remove the expect
                            let address = addresses.iter().next().expect("no addresses returned!");

                            // TODO move this part in his own method
                            let pinger = tokio_ping::Pinger::new();
                            let stream = pinger.and_then(move |pinger| Ok(pinger.chain(address).stream()));
                            let future = stream.and_then(|stream| {
                                stream.take(3).for_each(|mb_time| {
                                    match mb_time {
                                        Some(time) => info!("time={}", time), // TODO register the result in warp10
                                        None => info!("timeout"), // TODO register the result in warp10
                                    }
                                    Ok(())
                                })
                            });

                            tokio::spawn(future.map_err(|err| { // we have to use map_err to avoid a type conflict between tokio and tokio-ping
                                error!("Error: {}", err)
                            }));
                        };
                    });
                },
            };
        };
    });
}
