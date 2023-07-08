#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error(transparent)]
    IO(
        #[backtrace]
        #[from]
        std::io::Error,
    ),
    #[error(transparent)]
    Json(
        #[backtrace]
        #[from]
        serde_json::Error,
    ),
}
