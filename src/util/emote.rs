pub fn get_splitter() -> String {
    return dotenvy::var("EMOTE_SPLITTER").expect("EMOTE_SPLITTER must be set");
}