use mongodb::{bson::{doc, Document}, options::FindOneAndUpdateOptions, results::InsertOneResult, Collection, Database};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{routes::ApiError, util::{base62::base62_encode, emote::get_splitter}};
use reqwest::header;

use super::DatabaseError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EmoteProvider {
    BTTV
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmoteProviderData {
    pub provider: EmoteProvider,
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Emote {
    pub _id: String,
    pub provider: EmoteProviderData,
    pub image_type: String,
    pub animated: bool,
    pub banned: bool,

    pub legacy_id: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct LegacyGlobalId {
    pub emoteName: String,
    pub emoteId: String
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct LegacyEmote {
    pub globalId: LegacyGlobalId,
    pub bttvId: String,
    pub name: String,
    pub imageType: String,
    pub banned: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmoteIdCounter {
    pub _id: String,
    pub counter: u64
}

impl Emote {
    pub async fn generate_id(db: &Database, name: &str) -> Result<String, DatabaseError> {
        let counter: Collection<EmoteIdCounter> = db.collection("emote_id_counters");

        let result = counter.find_one_and_update(
            doc! {
                "_id": name
            },
            doc! {
                "$inc": {
                    "counter": 1
                }
            },
            FindOneAndUpdateOptions::builder().upsert(true).build()
        ).await.map_err(|e| DatabaseError::Database(e))?;

        let splitter = get_splitter();
        if result.is_none() {
            return Ok(format!("{}{}{}", name, &splitter, base62_encode(0)))
        }

        Ok(format!("{}{}{}", name, &splitter, base62_encode(result.as_ref().unwrap().counter)))
    }

    pub async fn find(db: &Database, doc: Document) -> Result<Option<Self>, DatabaseError> {
        let collection: Collection<Self> = db.collection("emotes");

        let existing_emote = collection.find_one(
            doc,
            None
        ).await.map_err(|e| DatabaseError::Database(e))?;

        Ok(existing_emote)
    }

    pub async fn insert(&self, db: &Database) -> Result<InsertOneResult, DatabaseError> {
        let collection: Collection<Self> = db.collection("emotes");
    
        Ok(collection.insert_one(self, None).await.map_err(|e| DatabaseError::Database(e))?)
    }
    

    pub async fn from_legacy_id(db: &Database, legacy_id: &str) -> Result<Self, ApiError> {
        let existing_emote = Self::find(db, doc! {
            "legacy_id": {
                "$eq": legacy_id
            }
        }).await.map_err(|e| ApiError::Database(e))?;

        if let Some(emote) = existing_emote {
            return Ok(emote);
        }

        let client = reqwest::Client::new();

        let legacy_response: LegacyEmote = client.get(format!(
                "{}/emote/get/",
                dotenvy::var("LEGACY_FALLBACK").unwrap_or("https://api.emotechat.de".to_string())
            ).to_owned() + legacy_id)
            .header(header::USER_AGENT, "LabyMod 4 EmoteChat API (https://neo.emotechat.de)")
            .send()
            .await
            .unwrap()
            .json::<LegacyEmote>()
            .await
            .map_err(|e| ApiError::LegacyApiError(e))?;

        let emote = Emote {
            _id: Self::generate_id(db, &legacy_response.name).await.unwrap(),
            provider: EmoteProviderData {
                provider: EmoteProvider::BTTV,
                id: legacy_response.bttvId
            },
            image_type: legacy_response.imageType.clone(),
            animated: legacy_response.imageType == "gif",
            banned: legacy_response.banned,
            legacy_id: Some(legacy_id.to_owned())
        };
        let _ = emote.insert(db).await.map(|_| emote.clone()).map_err(|e| ApiError::Database(e));            

        Ok(emote)
    }
}