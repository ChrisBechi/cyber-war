use crate::{error::GameResult, vfs::domain, world::WorldState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ForumState {
    threads: Vec<ForumThread>,
    replies: BTreeMap<String, Vec<ForumPost>>,
    followed: BTreeSet<String>,
    read_counts: BTreeMap<String, usize>,
    reports: Vec<ForumReport>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ForumThread {
    id: String,
    title: String,
    author: String,
    body: String,
    category: String,
    created_at: String,
    #[serde(default)]
    locked: bool,
    #[serde(default)]
    required_flag: Option<String>,
    #[serde(default)]
    replies: Vec<SeedReply>,
}

#[derive(Clone, Deserialize, Serialize)]
struct SeedReply {
    author: String,
    text: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ForumPost {
    id: String,
    author: String,
    text: String,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    quote: Option<SeedReply>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ForumReport {
    thread_id: String,
    post_id: String,
    reason: String,
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ForumAction {
    CreateThread {
        category: String,
        title: String,
        body: String,
    },
    Reply {
        thread_id: String,
        body: String,
        quote_id: Option<String>,
    },
    ToggleFollow {
        thread_id: String,
    },
    MarkRead {
        thread_id: String,
    },
    MarkAllRead,
    Report {
        thread_id: String,
        post_id: String,
        reason: String,
    },
}

#[derive(Deserialize)]
struct Community {
    categories: Vec<Category>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Category {
    id: String,
    read_only: bool,
}

fn threads(state: &ForumState, world: &WorldState) -> GameResult<Vec<ForumThread>> {
    let mut result: Vec<ForumThread> =
        serde_json::from_str(include_str!("../../content/forums/threads.json"))?;
    result.extend(state.threads.clone());
    result.retain(|thread| {
        thread
            .required_flag
            .as_ref()
            .is_none_or(|flag| world.flags.contains(flag))
    });
    Ok(result)
}

fn find_thread(state: &ForumState, world: &WorldState, id: &str) -> GameResult<ForumThread> {
    threads(state, world)?
        .into_iter()
        .find(|thread| thread.id == id)
        .ok_or_else(|| domain("Este tópico não está disponível."))
}

fn post_count(state: &ForumState, thread: &ForumThread) -> usize {
    1 + thread.replies.len() + state.replies.get(&thread.id).map_or(0, Vec::len)
}

fn find_post(state: &ForumState, thread: &ForumThread, id: &str) -> GameResult<SeedReply> {
    if id == format!("{}:op", thread.id) {
        return Ok(SeedReply {
            author: thread.author.clone(),
            text: thread.body.clone(),
        });
    }
    for (index, reply) in thread.replies.iter().enumerate() {
        if id == format!("{}:reply:{index}", thread.id) {
            return Ok(reply.clone());
        }
    }
    state
        .replies
        .get(&thread.id)
        .and_then(|posts| posts.iter().find(|post| post.id == id))
        .map(|post| SeedReply {
            author: post.author.clone(),
            text: post.text.clone(),
        })
        .ok_or_else(|| domain("Mensagem não encontrada neste tópico."))
}

fn text(value: &str, minimum: usize, maximum: usize) -> GameResult<String> {
    let value = value.trim();
    if !(minimum..=maximum).contains(&value.chars().count())
        || value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r')
    {
        return Err(domain(format!(
            "Use entre {minimum} e {maximum} caracteres válidos."
        )));
    }
    Ok(value.to_owned())
}

pub fn apply(world: &mut WorldState, action: ForumAction) -> GameResult<Option<String>> {
    if !world.network.connected {
        return Err(domain("Conecte-se à rede para acessar o fórum."));
    }
    let mut state: ForumState = world
        .settings
        .get("forumState")
        .map(|json| serde_json::from_str(json))
        .transpose()?
        .unwrap_or_default();
    let mut created = None;
    match action {
        ForumAction::CreateThread {
            category,
            title,
            body,
        } => {
            let community: Community =
                serde_json::from_str(include_str!("../../content/forums/community.json"))?;
            if !community
                .categories
                .iter()
                .any(|item| item.id == category && !item.read_only)
            {
                return Err(domain("Esta seção não permite novos tópicos."));
            }
            if state.threads.len() >= 128 {
                return Err(domain("Limite de tópicos da campanha atingido."));
            }
            let id = format!("player-{}", uuid::Uuid::new_v4());
            state.threads.push(ForumThread {
                id: id.clone(),
                title: text(&title, 5, 120)?,
                author: world.nickname.clone(),
                body: text(&body, 10, 12000)?,
                category,
                created_at: chrono::Utc::now().to_rfc3339(),
                locked: false,
                required_flag: None,
                replies: Vec::new(),
            });
            state.followed.insert(id.clone());
            state.read_counts.insert(id.clone(), 1);
            created = Some(id);
        }
        ForumAction::Reply {
            thread_id,
            body,
            quote_id,
        } => {
            let thread = find_thread(&state, world, &thread_id)?;
            if thread.locked {
                return Err(domain("Este tópico está fechado para respostas."));
            }
            if state.replies.values().map(Vec::len).sum::<usize>() >= 512 {
                return Err(domain("Limite de respostas da campanha atingido."));
            }
            let quote = quote_id
                .map(|id| find_post(&state, &thread, &id))
                .transpose()?
                .map(|mut quote| {
                    quote.text = quote.text.chars().take(1000).collect();
                    quote
                });
            let post = ForumPost {
                id: format!("post-{}", uuid::Uuid::new_v4()),
                author: world.nickname.clone(),
                text: text(&body, 2, 12000)?,
                created_at: chrono::Utc::now().to_rfc3339(),
                quote,
            };
            created = Some(post.id.clone());
            state
                .replies
                .entry(thread_id.clone())
                .or_default()
                .push(post);
            state
                .read_counts
                .insert(thread_id, post_count(&state, &thread));
        }
        ForumAction::ToggleFollow { thread_id } => {
            find_thread(&state, world, &thread_id)?;
            if !state.followed.remove(&thread_id) {
                state.followed.insert(thread_id);
            }
        }
        ForumAction::MarkRead { thread_id } => {
            let thread = find_thread(&state, world, &thread_id)?;
            state
                .read_counts
                .insert(thread_id, post_count(&state, &thread));
        }
        ForumAction::MarkAllRead => {
            for thread in threads(&state, world)? {
                state
                    .read_counts
                    .insert(thread.id.clone(), post_count(&state, &thread));
            }
        }
        ForumAction::Report {
            thread_id,
            post_id,
            reason,
        } => {
            let thread = find_thread(&state, world, &thread_id)?;
            let post = find_post(&state, &thread, &post_id)?;
            if post.author == world.nickname {
                return Err(domain("Você não pode sinalizar sua própria mensagem."));
            }
            if state.reports.iter().any(|report| report.post_id == post_id) {
                return Err(domain("Esta mensagem já foi sinalizada."));
            }
            if state.reports.len() >= 128 {
                return Err(domain("Limite de sinalizações atingido."));
            }
            state.reports.push(ForumReport {
                thread_id,
                post_id,
                reason: text(&reason, 4, 500)?,
            });
        }
    }
    let json = serde_json::to_string(&state)?;
    if json.len() > 2 * 1024 * 1024 {
        return Err(domain("O fórum atingiu o limite de dados desta campanha."));
    }
    world.settings.insert("forumState".into(), json);
    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> WorldState {
        let mut world = WorldState::new("tester", "kali").unwrap();
        world.network.connected = true;
        world
    }

    #[test]
    fn posts_follow_the_campaign_and_quotes_use_the_original_author() {
        let mut world = world();
        let id = apply(
            &mut world,
            ForumAction::CreateThread {
                category: "linux".into(),
                title: "Minha dúvida sobre arquivos".into(),
                body: "Onde ficam os arquivos do projeto?".into(),
            },
        )
        .unwrap()
        .unwrap();
        apply(
            &mut world,
            ForumAction::Reply {
                thread_id: id.clone(),
                body: "Encontrei a pasta, obrigado.".into(),
                quote_id: Some(format!("{id}:op")),
            },
        )
        .unwrap();
        let (json, hash) = crate::save::encode(&world).unwrap();
        let restored = crate::save::decode(&json, &hash).unwrap();
        let state: ForumState = serde_json::from_str(&restored.settings["forumState"]).unwrap();
        assert_eq!(state.threads[0].author, "tester");
        assert_eq!(
            state.replies[&id][0].quote.as_ref().unwrap().author,
            "tester"
        );
        assert_eq!(state.read_counts[&id], 2);
        assert!(state.followed.contains(&id));
    }

    #[test]
    fn hidden_locked_and_disconnected_threads_cannot_be_modified() {
        let mut world = world();
        for id in ["release", "server", "welcome", "missing"] {
            assert!(apply(
                &mut world,
                ForumAction::Reply {
                    thread_id: id.into(),
                    body: "Teste de resposta".into(),
                    quote_id: None
                }
            )
            .is_err());
        }
        assert!(!world.settings.contains_key("forumState"));
        assert!(apply(
            &mut world,
            ForumAction::CreateThread {
                category: "announcements".into(),
                title: "Um anúncio".into(),
                body: "Conteúdo de anúncio".into()
            }
        )
        .is_err());
        world.flags.insert("V1_COMPLETE".into());
        assert!(apply(
            &mut world,
            ForumAction::Reply {
                thread_id: "release".into(),
                body: "Agora está disponível.".into(),
                quote_id: None
            }
        )
        .is_ok());
        world.network.connected = false;
        assert!(apply(&mut world, ForumAction::MarkAllRead).is_err());
    }

    #[test]
    fn invalid_quotes_and_duplicate_reports_do_not_change_saved_forum_state() {
        let mut world = world();
        apply(
            &mut world,
            ForumAction::Report {
                thread_id: "welcome".into(),
                post_id: "welcome:op".into(),
                reason: "Revisar este aviso".into(),
            },
        )
        .unwrap();
        let before = world.settings["forumState"].clone();
        assert!(apply(
            &mut world,
            ForumAction::Report {
                thread_id: "welcome".into(),
                post_id: "welcome:op".into(),
                reason: "Repetição".into()
            }
        )
        .is_err());
        assert!(apply(
            &mut world,
            ForumAction::Reply {
                thread_id: "night-shift".into(),
                body: "Uma resposta".into(),
                quote_id: Some("welcome:op".into())
            }
        )
        .is_err());
        assert_eq!(before, world.settings["forumState"]);
    }
}
