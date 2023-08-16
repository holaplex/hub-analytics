use holaplex_hub_analytics::{
    cube_client::Client,
    db::Connection,
    events,
    graphql::schema::build_schema,
    handlers::{graphql_handler, health, playground},
    AppState, Args, Services,
};
use hub_core::{
    prelude::*,
    tokio::{self, task},
};
use poem::{get, listener::TcpListener, middleware::AddData, post, EndpointExt, Route, Server};

pub fn main() {
    let opts = hub_core::StartConfig {
        service_name: "hub-analytics",
    };

    hub_core::run(opts, |common, args| {
        let Args { port, db, cube } = args;

        common.rt.block_on(async move {
            let connection = Connection::new(db)
                .await
                .context("failed to get database connection")?;

            let schema = build_schema();
            let cube_client = Client::from_args(&cube)?;
            let state = AppState::new(schema, connection.clone(), cube_client.clone());
            let cons = common.consumer_cfg.build::<Services>().await?;

            tokio::spawn(async move {
                {
                    let mut stream = cons.stream();
                    loop {
                        let connection = connection.clone();
                        match stream.next().await {
                            Some(Ok(msg)) => {
                                info!(?msg, "message received");

                                tokio::spawn(async move {
                                    events::process(msg, connection.clone()).await
                                });
                                task::yield_now().await;
                            },
                            None => (),
                            Some(Err(e)) => {
                                warn!("failed to get message {:?}", e);
                            },
                        }
                    }
                }
            });

            Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
                .run(
                    Route::new()
                        .at("/graphql", post(graphql_handler).with(AddData::new(state)))
                        .at("/playground", get(playground))
                        .at("/health", get(health)),
                )
                .await
                .context("failed to build graphql server")
        })
    });
}
