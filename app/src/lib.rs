#![deny(clippy::disallowed_methods, clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

pub mod cube_client;
pub mod db;
#[allow(clippy::pedantic)]
pub mod entities;
pub mod events;
pub mod graphql;
pub mod handlers;
use db::Connection;
use hub_core::{clap, consumer::RecvError, prelude::*, uuid::Uuid};
use poem::{async_trait, FromRequest, Request, RequestBody};
#[allow(clippy::pedantic)]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/organization.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/customer.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/treasury.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/credential.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/webhook.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/nfts.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/solana_nfts.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/polygon_nfts.proto.rs"));
}

#[derive(Debug)]
pub enum Services {
    Organizations(proto::OrganizationEventKey, proto::OrganizationEvents),
    Customers(proto::CustomerEventKey, proto::CustomerEvents),
    Treasuries(proto::TreasuryEventKey, proto::TreasuryEvents),
    Nfts(proto::NftEventKey, proto::NftEvents),
    SolanaNfts(proto::SolanaNftEventKey, proto::SolanaNftEvents),
}

impl hub_core::consumer::MessageGroup for Services {
    const REQUESTED_TOPICS: &'static [&'static str] = &[
        "hub-orgs",
        "hub-customers",
        "hub-treasuries",
        "hub-nfts",
        "hub-nfts-solana",
        "hub-nfts-polygon",
    ];

    fn from_message<M: hub_core::consumer::Message>(msg: &M) -> Result<Self, RecvError> {
        let topic = msg.topic();
        let key = msg.key().ok_or(RecvError::MissingKey)?;
        let val = msg.payload().ok_or(RecvError::MissingPayload)?;
        info!(topic, ?key, ?val);

        match topic {
            "hub-orgs" => {
                let key = proto::OrganizationEventKey::decode(key)?;
                let val = proto::OrganizationEvents::decode(val)?;

                Ok(Services::Organizations(key, val))
            },
            "hub-customers" => {
                let key = proto::CustomerEventKey::decode(key)?;
                let val = proto::CustomerEvents::decode(val)?;

                Ok(Services::Customers(key, val))
            },
            "hub-treasuries" => {
                let key = proto::TreasuryEventKey::decode(key)?;
                let val = proto::TreasuryEvents::decode(val)?;

                Ok(Services::Treasuries(key, val))
            },
            "hub-nfts" => {
                let key = proto::NftEventKey::decode(key)?;
                let val = proto::NftEvents::decode(val)?;

                Ok(Services::Nfts(key, val))
            },
            "hub-nfts-solana" => {
                let key = proto::SolanaNftEventKey::decode(key)?;
                let val = proto::SolanaNftEvents::decode(val)?;

                Ok(Services::SolanaNfts(key, val))
            },
            t => Err(RecvError::BadTopic(t.into())),
        }
    }
}

#[derive(Debug, clap::Args)]
#[command(version, author, about)]
pub struct Args {
    #[arg(short, long, env, default_value_t = 3008)]
    pub port: u16,

    #[command(flatten)]
    pub db: db::DbArgs,

    #[command(flatten)]
    pub cube: cube_client::CubeArgs,
}

#[derive(Debug, Clone, Copy)]
pub struct UserID(Option<Uuid>);

impl TryFrom<&str> for UserID {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        let id = Uuid::from_str(value)?;

        Ok(Self(Some(id)))
    }
}

#[async_trait]
impl<'a> FromRequest<'a> for UserID {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        let id = req
            .headers()
            .get("X-USER-ID")
            .and_then(|value| value.to_str().ok())
            .map_or(Ok(Self(None)), Self::try_from)?;

        Ok(id)
    }
}

#[derive(Clone)]
pub struct AppState {
    pub schema: graphql::schema::AppSchema,
    pub connection: Connection,
    pub cube: cube_client::Client,
}

impl AppState {
    #[must_use]
    pub fn new(
        schema: graphql::schema::AppSchema,
        connection: Connection,
        cube: cube_client::Client,
    ) -> Self {
        Self {
            schema,
            connection,
            cube,
        }
    }
}

pub struct AppContext {
    pub user_id: Option<Uuid>,
}

impl AppContext {
    #[must_use]
    pub fn new(user_id: Option<Uuid>) -> Self {
        Self { user_id }
    }
}
