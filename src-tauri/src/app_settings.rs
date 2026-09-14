use crate::{error::GameResult, vfs::domain};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub skip_studio_intro: bool,
    pub skip_trailer: bool,
    pub language: String,
    pub master: u8,
    pub music: u8,
    pub sfx: u8,
    pub voice: u8,
    pub fullscreen: bool,
    pub resolution: String,
    pub ui_scale: u16,
    pub reduced_motion: bool,
    pub high_contrast: bool,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            skip_studio_intro: false,
            skip_trailer: false,
            language: "pt-BR".into(),
            master: 80,
            music: 70,
            sfx: 80,
            voice: 80,
            fullscreen: true,
            resolution: "1440x900".into(),
            ui_scale: 100,
            reduced_motion: false,
            high_contrast: false,
        }
    }
}
impl AppSettings {
    pub fn validate(&self) -> GameResult<()> {
        if [self.master, self.music, self.sfx, self.voice]
            .iter()
            .any(|v| *v > 100)
            || self.language != "pt-BR"
            || !["1024x640", "1280x720", "1440x900", "1920x1080"]
                .contains(&self.resolution.as_str())
            || ![90, 100, 110, 125].contains(&self.ui_scale)
        {
            return Err(domain("Configurações globais inválidas"));
        }
        Ok(())
    }
}
pub fn load(connection: &Connection) -> GameResult<AppSettings> {
    let json: Option<String> = connection
        .query_row(
            "SELECT value FROM app_preferences WHERE key='global'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    let settings = json
        .map(|value| serde_json::from_str(&value))
        .transpose()?
        .unwrap_or_default();
    AppSettings::validate(&settings)?;
    Ok(settings)
}
pub fn save(connection: &Connection, settings: &AppSettings) -> GameResult<AppSettings> {
    settings.validate()?;
    connection.execute("INSERT INTO app_preferences(key,value) VALUES('global',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [serde_json::to_string(settings)?])?;
    Ok(settings.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn global_settings_roundtrip_without_campaign_and_reject_invalid_values() {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::db::migrate(&mut connection).unwrap();
        assert_eq!(load(&connection).unwrap(), AppSettings::default());
        let saved = AppSettings {
            skip_studio_intro: true,
            skip_trailer: true,
            master: 35,
            music: 0,
            sfx: 42,
            ..AppSettings::default()
        };
        save(&connection, &saved).unwrap();
        crate::db::migrate(&mut connection).unwrap();
        assert_eq!(load(&connection).unwrap(), saved);
        let invalid = AppSettings {
            master: 101,
            ..saved.clone()
        };
        assert!(save(&connection, &invalid).is_err());
        assert_eq!(load(&connection).unwrap(), saved);
        assert_eq!(crate::save::list(&connection).unwrap().len(), 5);
    }

    #[test]
    fn preferences_survive_database_reopen() {
        let path = std::env::temp_dir().join(format!(
            "game-hacker-preferences-{}.db",
            uuid::Uuid::new_v4()
        ));
        let saved = AppSettings {
            skip_studio_intro: true,
            skip_trailer: true,
            master: 23,
            music: 0,
            ..AppSettings::default()
        };
        {
            let mut connection = Connection::open(&path).unwrap();
            crate::db::migrate(&mut connection).unwrap();
            save(&connection, &saved).unwrap();
        }
        {
            let mut connection = Connection::open(&path).unwrap();
            crate::db::migrate(&mut connection).unwrap();
            assert_eq!(load(&connection).unwrap(), saved);
            assert!(crate::save::list(&connection)
                .unwrap()
                .iter()
                .all(|slot| !slot.occupied));
        }
        std::fs::remove_file(path).unwrap();
    }
}
