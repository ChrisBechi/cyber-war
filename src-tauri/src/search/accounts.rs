use crate::{error::GameResult, vfs::domain, world::WorldState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualGoggleAccount {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub password_virtual: String,
    pub created_at: u64,
    pub apps: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAccount {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub apps: Vec<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoggleSession {
    pub account: Option<PublicAccount>,
    pub history_enabled: bool,
    pub history: Vec<String>,
}
pub fn session(world: &WorldState) -> GoggleSession {
    let account = world
        .search
        .session
        .as_ref()
        .and_then(|id| world.search.accounts.get(id))
        .map(|a| PublicAccount {
            id: a.id.clone(),
            email: a.email.clone(),
            display_name: a.display_name.clone(),
            avatar: a.avatar.clone(),
            apps: a.apps.clone(),
        });
    GoggleSession {
        account,
        history_enabled: world.search.history_enabled,
        history: world.search.history.clone(),
    }
}
fn password(id: &str, value: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest(format!("goggle-virtual:{id}:{value}"))
    )
}
pub fn authenticate(
    world: &mut WorldState,
    email: &str,
    secret: &str,
    display_name: Option<&str>,
) -> GameResult<()> {
    let email = email.trim().to_ascii_lowercase();
    let local = email
        .strip_suffix("@goggle.com")
        .ok_or_else(|| domain("Use um endereço @goggle.com."))?;
    if local.is_empty()
        || local.len() > 40
        || !local
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        || !(4..=128).contains(&secret.len())
    {
        return Err(domain(
            "Use um endereço válido e uma senha fictícia de 4 a 128 caracteres.",
        ));
    }
    if let Some(name) = display_name {
        if world.search.accounts.values().any(|a| a.email == email) {
            return Err(domain("Esta conta já existe. Faça login."));
        }
        if name.trim().is_empty() || name.len() > 80 || name.chars().any(char::is_control) {
            return Err(domain("Nome inválido."));
        }
        if world.search.accounts.len() >= 20 {
            return Err(domain("Limite de contas desta campanha atingido."));
        }
        let id = uuid::Uuid::new_v4().to_string();
        world.search.accounts.insert(
            id.clone(),
            VirtualGoggleAccount {
                id: id.clone(),
                email,
                display_name: name.trim().into(),
                avatar: None,
                password_virtual: password(&id, secret),
                created_at: world.playtime_seconds,
                apps: vec!["images".into(), "account".into()],
            },
        );
        world.search.session = Some(id);
    } else {
        let account = world
            .search
            .accounts
            .values()
            .find(|a| a.email == email && a.password_virtual == password(&a.id, secret))
            .ok_or_else(|| domain("E-mail ou senha fictícia incorretos."))?;
        world.search.session = Some(account.id.clone());
    }
    Ok(())
}
