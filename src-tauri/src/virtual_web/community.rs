//! Social and person-to-person commerce are campaign-local. No external messages or payments.
use super::{model::*, now, repository, visible};
use crate::{error::GameResult, vfs::domain, world::WorldState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommunityState {
    pub following: BTreeSet<String>,
    pub posts: Vec<PersonalPost>,
    pub messages: Vec<Message>,
    pub read: BTreeSet<String>,
    pub listings: BTreeMap<String, ListingState>,
    pub sequence: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalPost {
    pub id: String,
    pub text: String,
    pub timestamp: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub target: String,
    pub text: String,
    pub incoming: bool,
    pub timestamp: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListingState {
    pub status: String,
    pub offer_cents: Option<u64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub url: String,
    pub read: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityPage {
    pub suggested: Vec<Card>,
    pub connections: Vec<Card>,
    pub following: Vec<String>,
    pub followers: usize,
    pub feed: Vec<Card>,
    pub posts: Vec<PersonalPost>,
    pub messages: Vec<Message>,
    pub notifications: Vec<Notification>,
    pub listing: Option<ListingState>,
    pub nickname: String,
}

pub fn validate(state: &CommunityState) -> GameResult<()> {
    if state.following.len() > 500
        || state.posts.len() > 100
        || state.messages.len() > 200
        || state.read.len() > 1000
        || state.listings.len() > 100
        || state.posts.iter().any(|p| !valid_text(&p.text))
        || state.messages.iter().any(|m| !valid_text(&m.text))
        || state
            .listings
            .values()
            .any(|l| !["AVAILABLE", "RESERVED", "SOLD"].contains(&l.status.as_str()))
    {
        return Err(domain("Estado da comunidade inválido."));
    }
    Ok(())
}
fn valid_text(text: &str) -> bool {
    !text.trim().is_empty()
        && text.chars().count() <= 1000
        && !text.chars().any(|c| c.is_control() && c != '\n')
}
pub fn listing_status(world: &WorldState, doc: &Document) -> Option<String> {
    if let Detail::Listing { status, .. } = &doc.detail {
        Some(
            world
                .web
                .community
                .listings
                .get(&doc.id)
                .map_or(status, |l| &l.status)
                .clone(),
        )
    } else {
        None
    }
}
pub fn page(world: &WorldState, brand: &Brand, doc: Option<&Document>) -> Option<CommunityPage> {
    if !matches!(brand.platform, Platform::Social | Platform::Classifieds) {
        return None;
    }
    let state = &world.web.community;
    let repo = repository();
    let mut feed: Vec<_> = repo
        .pack
        .documents
        .iter()
        .filter(|d| visible(world, d))
        .filter(|d| {
            if let Detail::SocialPost { profile_id, .. } = &d.detail {
                doc.filter(|d| matches!(d.detail, Detail::SocialProfile { .. }))
                    .map_or_else(
                        || state.following.contains(profile_id),
                        |p| &p.id == profile_id,
                    )
            } else {
                false
            }
        })
        .collect();
    feed.sort_by_key(|d| (std::cmp::Reverse(d.published_at), &d.id));
    let notifications = feed
        .iter()
        .take(40)
        .map(|d| Notification {
            id: d.id.clone(),
            title: d.title.clone(),
            url: repo.url(d),
            read: state.read.contains(&d.id),
        })
        .collect();
    let followers = doc.map_or(0, |doc| {
        repo.pack.documents.iter().filter(|d| visible(world,d)).filter(|d|
        matches!(&d.detail, Detail::SocialProfile { following, .. } if following.contains(&doc.id))
    ).count() + usize::from(state.following.contains(&doc.id))
    });
    Some(CommunityPage {
        suggested: repo
            .pack
            .documents
            .iter()
            .filter(|d| {
                matches!(d.detail, Detail::SocialProfile { .. })
                    && visible(world, d)
                    && !state.following.contains(&d.id)
            })
            .take(6)
            .map(|d| repo.card(world, d))
            .collect(),
        connections: doc
            .and_then(|d| {
                if let Detail::SocialProfile { following, .. } = &d.detail {
                    Some(following)
                } else {
                    None
                }
            })
            .into_iter()
            .flatten()
            .filter_map(|id| repo.document(id))
            .filter(|d| visible(world, d))
            .map(|d| repo.card(world, d))
            .collect(),
        following: state
            .following
            .iter()
            .filter(|id| repo.document(id).is_some_and(|d| visible(world, d)))
            .cloned()
            .collect(),
        followers,
        feed: feed
            .into_iter()
            .take(30)
            .map(|d| repo.card(world, d))
            .collect(),
        posts: state
            .posts
            .iter()
            .filter(|p| p.timestamp <= now(world))
            .cloned()
            .collect(),
        messages: state
            .messages
            .iter()
            .filter(|m| {
                m.timestamp <= now(world)
                    && repo
                        .document(&m.target)
                        .is_some_and(|d| visible(world, d) && d.brand_id == brand.id)
                    && doc.is_none_or(|d| d.path == "/" || m.target == d.id)
            })
            .cloned()
            .collect(),
        notifications,
        nickname: world.nickname.clone(),
        listing: doc.and_then(|d| {
            if let Detail::Listing { status, .. } = &d.detail {
                Some(state.listings.get(&d.id).cloned().unwrap_or(ListingState {
                    status: status.clone(),
                    offer_cents: None,
                }))
            } else {
                None
            }
        }),
    })
}
fn message(world: &mut WorldState, id: &str, text: String, incoming: bool) {
    let timestamp = now(world);
    let state = &mut world.web.community;
    state.sequence = state.sequence.saturating_add(1);
    state.messages.push(Message {
        id: format!("message-{}", state.sequence),
        target: id.into(),
        text,
        incoming,
        timestamp,
    });
    if state.messages.len() > 200 {
        state.messages.drain(..state.messages.len() - 200);
    }
}
pub fn interact(
    world: &mut WorldState,
    doc: &Document,
    action: &str,
    text: &str,
) -> GameResult<bool> {
    let id = &doc.id;
    match action {
        "follow" if matches!(doc.detail, Detail::SocialProfile { .. }) => {
            if !world.web.community.following.remove(id) {
                if world.web.community.following.len() >= 500 {
                    return Err(domain("Limite de conexões atingido."));
                }
                world.web.community.following.insert(id.clone());
            }
        }
        "post" if repository().brand(&doc.brand_id).platform == Platform::Social => {
            if !valid_text(text) {
                return Err(domain("Escreva até 1.000 caracteres."));
            }
            if world.web.community.posts.len() >= 100 {
                return Err(domain(
                    "Limite de publicações atingido. Remova uma publicação antes de continuar.",
                ));
            }
            let timestamp = now(world);
            let state = &mut world.web.community;
            state.sequence = state.sequence.saturating_add(1);
            state.posts.insert(
                0,
                PersonalPost {
                    id: format!("post-{}", state.sequence),
                    text: text.trim().into(),
                    timestamp,
                },
            );
        }
        "deletePost" if repository().brand(&doc.brand_id).platform == Platform::Social => {
            world.web.community.posts.retain(|p| p.id != text);
        }
        "readNotification" if matches!(doc.detail, Detail::SocialPost { .. }) => {
            if world.web.community.read.len() >= 1000 {
                world.web.community.read.clear();
            }
            world.web.community.read.insert(id.clone());
        }
        "message"
            if matches!(
                doc.detail,
                Detail::SocialProfile { .. } | Detail::Listing { .. }
            ) =>
        {
            if !valid_text(text) {
                return Err(domain("Escreva uma mensagem com até 1.000 caracteres."));
            }
            message(world, id, text.trim().into(), false);
            let response = match &doc.detail {
                Detail::SocialProfile { role, employer, .. } => format!("Obrigado pelo contato! Trabalho com {role}. Meu vínculo profissional e os projetos públicos estão em {}. Mensagens sobre vagas são recebidas pela equipe da empresa.", employer.label),
                Detail::Listing { .. } if listing_status(world, doc).as_deref() == Some("SOLD") => "Esse item já foi vendido. Obrigado pelo interesse!".into(),
                Detail::Listing { .. } if listing_status(world, doc).as_deref() == Some("RESERVED") => "O item está reservado. A retirada pode ser combinada por aqui; a confirmação encerra o anúncio.".into(),
                Detail::Listing { condition, location, .. } => format!("O item está {condition} e fica em {location}. Podemos conferir juntos antes da retirada. Use Enviar proposta para combinar o valor; a conversa não solicita transferência antecipada."),
                _ => unreachable!(),
            };
            message(world, id, response, true);
        }
        "offer" | "reserve" | "release" | "sold"
            if matches!(doc.detail, Detail::Listing { .. }) =>
        {
            let Detail::Listing {
                price_cents,
                minimum_cents,
                status,
                ..
            } = &doc.detail
            else {
                unreachable!()
            };
            let current = world
                .web
                .community
                .listings
                .get(id)
                .cloned()
                .unwrap_or(ListingState {
                    status: status.clone(),
                    offer_cents: None,
                });
            if world.web.community.listings.len() >= 100
                && !world.web.community.listings.contains_key(id)
            {
                return Err(domain("Limite de negociações atingido."));
            }
            if current.status == "SOLD" {
                return Err(domain("Este anúncio já foi vendido."));
            }
            let mut next = current.clone();
            match action {
                "offer" => {
                    if current.status != "AVAILABLE" {
                        return Err(domain("Libere a reserva antes de alterar a proposta."));
                    }
                    let cents: u64 = text
                        .parse()
                        .map_err(|_| domain("Valor da proposta inválido."))?;
                    if cents == 0 || cents > *price_cents {
                        return Err(domain(
                            "A proposta deve ser positiva e não superar o preço anunciado.",
                        ));
                    }
                    if cents < *minimum_cents {
                        message(world,id,"Obrigado pela proposta. Esse valor ficou abaixo do que consigo aceitar; podemos conversar por um valor mais próximo do anúncio.".into(),true);
                    } else {
                        next.offer_cents = Some(cents);
                        message(world,id,format!("Proposta de R$ {:.2} aceita. Você pode reservar o item e combinar a retirada.",cents as f64/100.),true);
                    }
                }
                "reserve" if current.status == "AVAILABLE" => {
                    next.status = "RESERVED".into();
                }
                "release" if current.status == "RESERVED" => {
                    next.status = "AVAILABLE".into();
                }
                "sold" if current.status == "RESERVED" => {
                    next.status = "SOLD".into();
                }
                _ => return Err(domain("Transição de anúncio inválida.")),
            }
            world.web.community.listings.insert(id.clone(), next);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
