use mongodb::{Client, Database, error::Error};

pub async fn connect() -> Result<Database, Error>{
    let client = Client::with_uri_str(
        &dotenvy::var("MONGO_URI").expect("MONGO_URI must be set")
    ).await.unwrap();

    let db = client.database(&dotenvy::var("MONGO_DATABASE").expect("MONGO_DATABASE must be set"));

    Ok(db)
}