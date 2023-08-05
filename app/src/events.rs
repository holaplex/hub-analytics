use hub_core::{chrono::Utc, prelude::*, uuid::Uuid};
use sea_orm::{prelude::*, Set};

use crate::{
    db::Connection,
    entities::{collections, customers, mints, organizations, projects, wallets},
    proto::{customer_events, nft_events, organization_events, solana_nft_events, treasury_events},
    Services,
};

/// Res
///
/// # Errors
/// This function fails if ...
#[allow(clippy::too_many_lines)]
pub async fn process(msg: Services, db: Connection) -> Result<()> {
    match msg {
        Services::Organizations(k, v) => match v.event {
            Some(organization_events::Event::OrganizationCreated(v)) => {
                organizations::ActiveModel {
                    id: Set(Uuid::parse_str(&k.id)?),
                    name: Set(v.name),
                }
                .insert(db.get())
                .await?;
                Ok(())
            },
            Some(organization_events::Event::ProjectCreated(v)) => {
                projects::ActiveModel {
                    id: Set(Uuid::parse_str(&k.id)?),
                    name: Set(v.name),
                    organization_id: Set(Uuid::parse_str(&v.organization_id)?),
                }
                .insert(db.get())
                .await?;
                Ok(())
            },
            Some(_) | None => Ok(()),
        },
        Services::Customers(k, v) => match v.event {
            Some(customer_events::Event::Created(v)) => {
                customers::ActiveModel {
                    id: Set(Uuid::parse_str(&k.id)?),
                    project_id: Set(Uuid::parse_str(&v.project_id)?),
                    timestamp: Set(Utc::now().naive_utc()),
                }
                .insert(db.get())
                .await?;
                Ok(())
            },
            Some(_) | None => Ok(()),
        },

        Services::Treasuries(k, v) => match v.event {
            Some(treasury_events::Event::CustomerWalletCreated(v)) => {
                wallets::ActiveModel {
                    id: Set(Uuid::parse_str(&k.id)?),
                    project_id: Set(Uuid::parse_str(&k.project_id)?),
                    blockchain: Set(int_to_blockchain(v.blockchain)),
                    timestamp: Set(Utc::now().naive_utc()),
                }
                .insert(db.get())
                .await?;
                Ok(())
            },
            Some(_) | None => Ok(()),
        },
        Services::Nfts(k, v) => match v.event {
            Some(nft_events::Event::SolanaCreateDrop(v)) => {
                collections::ActiveModel {
                    id: Set(Uuid::parse_str(&k.id)?),
                    name: Set(v.master_edition.unwrap_or_default().name),
                    project_id: Set(Uuid::parse_str(&k.project_id)?),
                    blockchain: Set("Solana".to_string()),
                    timestamp: Set(Utc::now().naive_utc()),
                }
                .insert(db.get())
                .await?;
                Ok(())
            },
            Some(nft_events::Event::PolygonCreateDrop(v)) => {
                collections::ActiveModel {
                    id: Set(Uuid::parse_str(&k.id)?),
                    name: Set(v.edition_info.unwrap_or_default().collection),
                    project_id: Set(Uuid::parse_str(&k.project_id)?),
                    blockchain: Set("Polygon".to_string()),
                    timestamp: Set(Utc::now().naive_utc()),
                }
                .insert(db.get())
                .await?;
                Ok(())
            },
            Some(nft_events::Event::DropMinted(v)) => {
                mints::ActiveModel {
                    id: Set(Uuid::parse_str(&k.id)?),
                    collection_id: Set(Uuid::parse_str(&v.drop_id)?),
                    project_id: Set(Uuid::parse_str(&k.project_id)?),
                    timestamp: Set(Utc::now().naive_utc()),
                }
                .insert(db.get())
                .await?;
                Ok(())
            },
            Some(_) | None => Ok(()),
        },
        Services::SolanaNfts(k, v) => match v.event {
            Some(solana_nft_events::Event::ImportedExternalCollection(v)) => {
                collections::ActiveModel {
                    id: Set(Uuid::parse_str(&k.id)?),
                    name: Set(v.metadata.unwrap_or_default().name),
                    project_id: Set(Uuid::parse_str(&k.project_id)?),
                    blockchain: Set("Solana".to_string()),
                    timestamp: Set(Utc::now().naive_utc()),
                }
                .insert(db.get())
                .await?;
                Ok(())
            },
            Some(solana_nft_events::Event::ImportedExternalMint(v)) => {
                mints::ActiveModel {
                    id: Set(Uuid::parse_str(&k.id)?),
                    collection_id: Set(Uuid::parse_str(&v.collection_id)?),
                    project_id: Set(Uuid::parse_str(&k.project_id)?),
                    timestamp: Set(Utc::now().naive_utc()),
                }
                .insert(db.get())
                .await?;
                Ok(())
            },
            Some(_) | None => Ok(()),
        },
    }
}

fn int_to_blockchain(n: i32) -> String {
    match n {
        1 => "Solana",
        2 => "Polygon",
        3 => "Ethereum",
        _ => "Unspecified",
    }
    .to_string()
}
