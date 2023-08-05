//!

use holaplex_hub_analytics::{db::Connection, events, Args, Services};
use hub_core::{
    prelude::*,
    tokio::{self, task},
};

pub fn main() {
    let opts = hub_core::StartConfig {
        service_name: "hub-orgs",
    };

    hub_core::run(opts, |common, args| {
        let Args { db } = args;

        common.rt.block_on(async move {
            let cons = common.consumer_cfg.build::<Services>().await?;
            let connection = Connection::new(db)
                .await
                .context("failed to get database connection")?;
            let mut stream = cons.stream();
            loop {
                let connection = connection.clone();
                match stream.next().await {
                    Some(Ok(msg)) => {
                        info!(?msg, "message received");

                        tokio::spawn(async move { events::process(msg, connection.clone()).await });
                        task::yield_now().await;
                    },
                    None => (),
                    Some(Err(e)) => {
                        warn!("failed to get message {:?}", e);
                    },
                }
            }
        })
    });
}
