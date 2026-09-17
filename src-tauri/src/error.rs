use thiserror::Error;

#[derive(Debug, Error)]
pub enum GameError {
    #[error("{0}")]
    Vfs(crate::vfs::Errno),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("invalid save slot: {0}; expected 1..=5")]
    InvalidSaveSlot(i64),

    #[error("save slot {0} is empty")]
    EmptySaveSlot(i64),

    #[error("{0}")]
    Domain(String),
}

impl serde::Serialize for GameError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type GameResult<T> = Result<T, GameError>;
