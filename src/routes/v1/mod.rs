use axum::Router;

mod emotes;

pub fn config() -> Router {
    Router::new().nest(
        "/v1",
        Router::new()
            .nest("/emote", emotes::config())
    )
}