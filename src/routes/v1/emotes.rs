use axum::{extract::Path, routing::{get, post}, Extension, Json, Router};
use mongodb::{bson::{doc, to_bson}, Collection, Database};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use crate::{
    database::models::{
        emote::{
            Emote, 
            EmoteProvider, EmoteProviderData
        },
        DatabaseError
    }, 
    routes::ApiError, util::emote::get_splitter,
};

pub fn config() -> Router {
    Router::new()
        .route("/:id", get(get_emote))
        .route("/", post(add_emote))       
}

pub fn validate_id(id: &str) -> bool {
    for c in id.chars() {
        if !(c.is_ascii_lowercase() || c.is_ascii_uppercase() || c.is_ascii_digit()) {
            return false;
        }
    }
    true
}

fn validate_legacy_id(raw_id: &str) -> bool {
    let mut has_lowercase = false;
    let mut splitter_index: Option<usize> = None;

    for (i, c) in raw_id.chars().enumerate() {
        if c.is_lowercase() {
            has_lowercase = true;
        }

        if c.is_uppercase() && splitter_index.is_none() {
            if !has_lowercase {
                return false;
            }
            splitter_index = Some(i);
        } else if (c.is_lowercase() || !c.is_alphabetic()) && splitter_index.is_some() {
            return false;
        }
    }

    splitter_index.is_some()
}

// /v1/emote/:id
async fn get_emote(Path(id): Path<String>, db: Extension<Database>) -> Result<Json<Emote>, ApiError> {
    let splitter = get_splitter();

    if !id.contains(&splitter) || !validate_id(id.split(&splitter).nth(1).unwrap()) {
        if !validate_legacy_id(&id.clone()) {
            return Err(ApiError::InvalidId);
        }
        
        return Ok(Json(Emote::from_legacy_id(&db, &id).await.map_err(|e| e)?));
    }

    let emote = Emote::find(&db, doc! {
        "_id": {
            "$eq": id
        }
    }).await.map_err(|e| ApiError::Database(e))?;

    Ok(Json(emote.ok_or(ApiError::NotFound)?))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmoteAddData {
    pub provider: EmoteProvider,
    pub id: String, // ID of the emote on the provider
}

impl EmoteAddData {
    pub fn emote_url(&self) -> String {
        match self.provider {
            EmoteProvider::BTTV => format!("https://api.betterttv.net/3/emotes/{}", self.id),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BTTVEmote {
    pub id: String,
    pub code: String,
    pub imageType: String,
    pub animated: bool,
}

// /v1/emote/add
async fn add_emote(
    Extension(db): Extension<Database>,
    Json(payload): Json<EmoteAddData>
) -> Result<Json<Emote>, ApiError> {
    // Check if emote already exists
    let collection: Collection<Emote> = db.collection("emotes");
    let filter = doc! {
        "provider.provider": { "$eq": to_bson(&payload.provider).unwrap() },
        "provider.id": { "$eq": to_bson(&payload.id).unwrap() }
    };

    let existing_emote = collection.find_one(filter, None).await.map_err(|e| ApiError::Database(DatabaseError::Database(e)))?;

    if let Some(emote) = existing_emote {
        return Ok(Json(emote));
    }

    // Fetch emote from provider
    let response = reqwest::get(payload.emote_url()).await.map_err(|e| ApiError::EmoteProviderError(e.to_string()))?;
    if !response.status().is_success() {
        return Err(ApiError::EmoteProviderError(format!("Emote provider returned status code: {}", response.status().as_u16())))
    }

    let bttv_emote: BTTVEmote = response.json().await.map_err(|e| ApiError::EmoteProviderError(e.to_string()))?;

    // Incremental ID generation test purposes
    // let bttv_emote = BTTVEmote {
    //     id: "5f5f4b7b6b8c5c372f7b23e3".to_string(),
    //     code: "monkaW".to_string(),
    //     imageType: "gif".to_string(),
    //     animated: true
    // };

    // Insert emote into database
    let new_emote = Emote {
        _id: Emote::generate_id(&db, &bttv_emote.code).await.map_err(|e| ApiError::Database(e))?,
        provider: EmoteProviderData {
            provider: payload.provider.clone(),
            id: payload.id.clone()
        },
        image_type: bttv_emote.imageType,
        banned: false,
        animated: bttv_emote.animated,
        legacy_id: None
    };
    new_emote.insert(&db).await.map_err(|e| ApiError::Database(e))?;

    Ok(Json(new_emote))
}