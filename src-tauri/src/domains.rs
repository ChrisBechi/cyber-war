//! A small, data-driven domain market that lives entirely inside the campaign world.
use crate::{error::GameResult, vfs::domain, world::WorldState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const RENEWAL_CYCLE_SECONDS: u64 = 3600;
const EXPIRING_WINDOW_SECONDS: u64 = 300;
const EXPIRY_GRACE_SECONDS: u64 = 900;
const SUBDOMAIN_SETUP_COST: i64 = 5;
pub const BLACKWIRE_ONION_ADDRESS: &str =
    "pg6mmjiyjmcrsslvykfwnntlaru7p5svn6y2ymmju6nubxndf4pscryd.onion";
const ONION_EXTENSION: &str = ".onion";
const ONION_LABEL_LENGTH: usize = 56;
const ONION_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz234567";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDefinition {
    pub suffix: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub category: String,
    pub country: Option<String>,
    pub registration_restricted: bool,
    pub allowed_business_types: Vec<String>,
    pub base_price: i64,
    pub renewal_price: i64,
    pub international_reputation: i32,
    pub technology_reputation: i32,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubdomainRecord {
    pub label: String,
    pub address: String,
    pub created_at_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainRecord {
    pub address: String,
    pub registered_name: String,
    pub suffix: String,
    pub category: String,
    pub country: Option<String>,
    pub owner: String,
    pub owner_kind: String,
    pub organization: String,
    pub business_type: String,
    pub registered_at_seconds: u64,
    pub expires_at_seconds: u64,
    pub auto_renew: bool,
    pub primary: bool,
    pub redirect_to: Option<String>,
    #[serde(default)]
    pub listed_price: Option<i64>,
    #[serde(default)]
    pub subdomains: BTreeMap<String, SubdomainRecord>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMarketOffer {
    pub id: String,
    pub address: String,
    pub buyer: String,
    pub amount: i64,
    pub status: String,
    pub counter_price: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DomainState {
    pub registrations: BTreeMap<String, DomainRecord>,
    pub offers: Vec<DomainMarketOffer>,
    #[serde(default = "seeded_onion_services")]
    pub onion_services: BTreeMap<String, OnionServiceRecord>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceRecord {
    pub address: String,
    pub service_name: String,
    pub description: String,
    pub owner: String,
    pub availability_mode: String,
    pub last_seen_seconds: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceInfo {
    pub address: String,
    pub service_name: String,
    pub description: String,
    pub owner: String,
    pub network: String,
    pub kind: String,
    #[serde(rename = "type")]
    pub address_type: String,
    pub extension: String,
    pub address_version: u8,
    pub address_length: usize,
    pub uses_traditional_dns: bool,
    pub registrable: bool,
    pub purchasable: bool,
    pub tradable: bool,
    pub generated_from_cryptographic_identity: bool,
    pub allowed_characters: String,
    pub online: bool,
    pub availability_mode: String,
    pub availability: String,
    pub last_seen_seconds: u64,
    pub next_change_seconds: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainOffer {
    pub address: String,
    pub registered_name: String,
    pub suffix: String,
    pub category: String,
    pub country: Option<String>,
    pub description: String,
    pub price: i64,
    pub renewal_price: i64,
    pub available: bool,
    pub restricted: bool,
    pub restriction_reason: Option<String>,
    pub premium: bool,
    pub recommendation: String,
    pub recommended: bool,
    pub status: String,
    pub owner: Option<String>,
    pub registered_at_year: Option<u32>,
    pub expires_at_year: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainSearchResult {
    pub query: String,
    pub normalized_name: String,
    pub results: Vec<DomainOffer>,
    pub suggestions: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainWhois {
    pub address: String,
    pub ip: Option<String>,
    pub status: String,
    pub owner: Option<String>,
    pub registered_at_year: Option<u32>,
    pub expires_at_year: Option<u32>,
    pub suffix: String,
    pub category: String,
    pub country: Option<String>,
    pub subdomains: Vec<String>,
    pub redirect_to: Option<String>,
    pub network: Option<String>,
    pub kind: Option<String>,
    pub uses_traditional_dns: Option<bool>,
    pub address_version: Option<u8>,
    pub registrable: Option<bool>,
    pub purchasable: Option<bool>,
    pub tradable: Option<bool>,
}

#[derive(Clone, Debug)]
struct ParsedDomain {
    address: String,
    name: String,
    suffix: String,
    subdomain: Option<String>,
    #[cfg_attr(not(test), allow(dead_code))]
    // Validated by parser tests; reserved domain metadata.
    jurisdiction: Option<String>,
    government_namespace: bool,
}

fn definitions() -> Vec<ExtensionDefinition> {
    serde_json::from_str(include_str!("../../content/domains/extensions.json"))
        .expect("valid domain extension catalog")
}

fn seeded_onion_services() -> BTreeMap<String, OnionServiceRecord> {
    [(
        BLACKWIRE_ONION_ADDRESS.into(),
        OnionServiceRecord {
            address: BLACKWIRE_ONION_ADDRESS.into(),
            service_name: "Blackwire Relay".into(),
            description: "Rede de espelhos anônimos para arquivos preservados.".into(),
            owner: "Blackwire mirror network".into(),
            availability_mode: "intermittent".into(),
            last_seen_seconds: 0,
        },
    )]
    .into_iter()
    .collect()
}

fn definition(suffix: &str) -> GameResult<ExtensionDefinition> {
    definitions()
        .into_iter()
        .find(|item| item.suffix == suffix)
        .ok_or_else(|| domain("Extensão de domínio não catalogada."))
}

fn valid_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn normalize_address(value: &str) -> GameResult<String> {
    let value = value.trim().to_lowercase();
    let value = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
        .unwrap_or(&value)
        .trim_end_matches('.');
    let value = value.strip_prefix("www.").unwrap_or(value);
    if value.is_empty() || value.len() > 253 || value.contains('/') || value.contains(':') {
        return Err(domain(
            "Domínio inválido: informe somente o endereço, sem protocolo ou caminho.",
        ));
    }
    let labels: Vec<_> = value.split('.').collect();
    if labels.iter().any(|label| !valid_label(label)) {
        return Err(domain(
            "Domínio inválido: use letras, números e hífens entre os pontos.",
        ));
    }
    Ok(value.into())
}

fn onion_host(value: &str) -> String {
    let value = value.trim();
    let value = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
        .unwrap_or(value);
    value
        .split('/')
        .next()
        .unwrap_or("")
        .trim_end_matches('.')
        .into()
}

pub fn looks_like_onion(value: &str) -> bool {
    onion_host(value)
        .to_ascii_lowercase()
        .ends_with(ONION_EXTENSION)
}

pub fn valid_onion_address(value: &str) -> bool {
    let host = onion_host(value);
    let Some(label) = host.strip_suffix(ONION_EXTENSION) else {
        return false;
    };
    label.len() == ONION_LABEL_LENGTH
        && !label.chars().all(|character| label.starts_with(character))
        && label
            .bytes()
            .all(|byte| ONION_ALPHABET.as_bytes().contains(&byte))
}

fn normalize_onion_address(value: &str) -> GameResult<String> {
    let host = onion_host(value);
    if !valid_onion_address(&host) {
        return Err(domain(
            "Endereço Onion inválido: use um serviço Tor v3 com exatamente 56 caracteres Base32 (a-z e 2-7).",
        ));
    }
    Ok(host)
}

fn onion_availability(record: &OnionServiceRecord, now: u64) -> (&'static str, bool, Option<u64>) {
    match record.availability_mode.as_str() {
        "always" => ("online", true, None),
        "intermittent" => {
            const CYCLE: u64 = 7 * 24 * 60 * 60;
            const ONLINE_WINDOW: u64 = 2 * 24 * 60 * 60;
            let phase = now % CYCLE;
            let online = phase < ONLINE_WINDOW;
            let next_change = if online {
                ONLINE_WINDOW - phase
            } else {
                CYCLE - phase
            };
            (
                if online { "online" } else { "offline" },
                online,
                Some(next_change),
            )
        }
        _ => ("offline", false, None),
    }
}

fn onion_info_from_record(record: &OnionServiceRecord, now: u64) -> OnionServiceInfo {
    let (availability, online, next_change_seconds) = onion_availability(record, now);
    OnionServiceInfo {
        address: record.address.clone(),
        service_name: record.service_name.clone(),
        description: record.description.clone(),
        owner: record.owner.clone(),
        network: "tor".into(),
        kind: "onion_service".into(),
        address_type: "special_use".into(),
        extension: ONION_EXTENSION.into(),
        address_version: 3,
        address_length: ONION_LABEL_LENGTH,
        uses_traditional_dns: false,
        registrable: false,
        purchasable: false,
        tradable: false,
        generated_from_cryptographic_identity: true,
        allowed_characters: ONION_ALPHABET.into(),
        online,
        availability_mode: record.availability_mode.clone(),
        availability: availability.into(),
        last_seen_seconds: record.last_seen_seconds,
        next_change_seconds,
    }
}

pub fn onion_service(world: &WorldState, address: &str) -> GameResult<OnionServiceInfo> {
    let address = normalize_onion_address(address)?;
    let record = world
        .domains
        .onion_services
        .get(&address)
        .ok_or_else(|| domain("Onion Service não encontrado na rede Tor."))?;
    Ok(onion_info_from_record(record, current(world)))
}

pub fn onion_online(world: &WorldState, address: &str) -> GameResult<OnionServiceInfo> {
    let info = onion_service(world, address)?;
    if !info.online {
        return Err(domain(format!(
            "Onion Service indisponível: o serviço está offline na rede Tor. Próxima mudança em {} segundos.",
            info.next_change_seconds.unwrap_or(0)
        )));
    }
    Ok(info)
}

fn parse(address: &str) -> GameResult<ParsedDomain> {
    let address = normalize_address(address)?;
    if address.ends_with(ONION_EXTENSION) {
        return Err(domain(
            "Endereços .onion são Onion Services da rede Tor e não participam do mercado tradicional.",
        ));
    }
    let labels: Vec<_> = address.split('.').collect();
    let mut matched: Option<(String, usize)> = None;
    for item in definitions() {
        let suffix_labels = item.suffix.trim_start_matches('.').split('.').count();
        if labels.len() > suffix_labels
            && address.ends_with(&item.suffix)
            && matched
                .as_ref()
                .is_none_or(|(_, count)| suffix_labels > *count)
        {
            matched = Some((item.suffix, suffix_labels));
        }
    }
    let Some((suffix, suffix_count)) = matched else {
        return Err(domain(
            "Extensão inválida ou ainda não disponível no mercado.",
        ));
    };
    let prefix = &labels[..labels.len() - suffix_count];
    if prefix.is_empty() {
        return Err(domain("Informe o nome registrado antes da extensão."));
    }
    let mut government_namespace = false;
    let (name, subdomain, jurisdiction) = if suffix == ".gov.br" {
        let state = prefix.last().copied();
        let states = [
            "ac", "al", "ap", "am", "ba", "ce", "df", "es", "go", "ma", "mg", "ms", "mt", "pa",
            "pb", "pe", "pi", "pr", "rj", "rn", "ro", "rr", "rs", "sc", "se", "sp", "to",
        ];
        if prefix.len() == 1 && state.is_some_and(|value| states.contains(&value)) {
            government_namespace = true;
            (prefix[0], None, None)
        } else if prefix.len() >= 2 && state.is_some_and(|value| states.contains(&value)) {
            let name_index = prefix.len() - 2;
            let subdomain = (name_index > 0).then(|| prefix[..name_index].join("."));
            (prefix[name_index], subdomain, state.map(str::to_owned))
        } else {
            (prefix[prefix.len() - 1], None, None)
        }
    } else {
        let name_index = prefix.len() - 1;
        let subdomain = (name_index > 0).then(|| prefix[..name_index].join("."));
        (prefix[name_index], subdomain, None)
    };
    let category = definition(&suffix)?.category;
    if government_namespace && category != "government" {
        return Err(domain("Estrutura de domínio governamental inválida."));
    }
    Ok(ParsedDomain {
        address: address.clone(),
        name: name.into(),
        suffix,
        subdomain,
        jurisdiction,
        government_namespace,
    })
}

fn current(world: &WorldState) -> u64 {
    world.playtime_seconds
}

fn status(record: &DomainRecord, now: u64) -> &'static str {
    if now
        < record
            .expires_at_seconds
            .saturating_sub(EXPIRING_WINDOW_SECONDS)
    {
        "active"
    } else if now < record.expires_at_seconds {
        "expiring"
    } else if now <= record.expires_at_seconds + EXPIRY_GRACE_SECONDS {
        "expired"
    } else {
        "available"
    }
}

fn year(seconds: u64) -> u32 {
    2028 + (seconds / RENEWAL_CYCLE_SECONDS) as u32
}

fn premium(name: &str) -> bool {
    matches!(
        name,
        "ai" | "car" | "shop" | "bank" | "food" | "app" | "tech"
    ) || name.len() <= 3
        || ["cloud", "pay", "market", "store", "health", "quantum"].contains(&name)
}

fn price(name: &str, extension: &ExtensionDefinition) -> i64 {
    if !premium(name) {
        return extension.base_price;
    }
    let multiplier = if name.len() <= 3 { 30 } else { 12 };
    extension.base_price.saturating_mul(multiplier)
}

fn normalized_name(query: &str) -> String {
    parse(query).map(|parsed| parsed.name).unwrap_or_else(|_| {
        query
            .trim()
            .to_lowercase()
            .chars()
            .filter(|char| char.is_ascii_alphanumeric() || *char == '-')
            .collect()
    })
}

fn recommendation(extension: &ExtensionDefinition, business_type: &str) -> String {
    match (business_type, extension.suffix.as_str()) {
        ("education", ".edu.br") => "Excelente para instituições de educação".into(),
        ("ngo" | "association" | "foundation" | "non_profit", ".org" | ".org.br") => {
            "Excelente para organizações e projetos sociais".into()
        }
        ("app", ".app") => "Excelente para aplicativos".into(),
        ("technology" | "startup", ".io" | ".tech") => {
            "Excelente para tecnologia e startups".into()
        }
        (_, ".com.br") => "Excelente para empresas brasileiras".into(),
        (_, ".com") => "Excelente para expansão internacional".into(),
        (_, ".net" | ".net.br") => "Boa escolha para redes e serviços online".into(),
        _ => "Uma opção coerente com a identidade do projeto".into(),
    }
}

fn restriction(extension: &ExtensionDefinition, business_type: &str) -> Option<String> {
    if extension.registration_restricted
        && !extension
            .allowed_business_types
            .iter()
            .any(|allowed| allowed == business_type)
    {
        return Some(
            "Este tipo de domínio possui requisitos específicos e não pode ser registrado por esta organização."
                .into(),
        );
    }
    None
}

fn offer(world: &WorldState, address: &str, business_type: &str) -> GameResult<DomainOffer> {
    let parsed = parse(address)?;
    let extension = definition(&parsed.suffix)?;
    let record = world.domains.registrations.get(&parsed.address);
    let now = current(world);
    let record_status = record.map(|item| status(item, now));
    let available = record.is_none_or(|_| record_status == Some("available"));
    let restriction_reason = if parsed.government_namespace {
        Some("Domínios governamentais são reservados a órgãos públicos.".into())
    } else {
        restriction(&extension, business_type)
    };
    let restricted = restriction_reason.is_some();
    let recommendation = recommendation(&extension, business_type);
    let domain_price = price(&parsed.name, &extension);
    Ok(DomainOffer {
        address: parsed.address,
        registered_name: parsed.name.clone(),
        suffix: parsed.suffix,
        category: extension.category,
        country: extension.country.clone(),
        description: extension.description.clone(),
        price: domain_price,
        renewal_price: extension.renewal_price,
        available: available && !restricted,
        restricted,
        restriction_reason,
        premium: premium(&parsed.name),
        recommended: matches!(
            (business_type, extension.suffix.as_str()),
            ("education", ".edu.br")
                | ("app", ".app")
                | ("technology" | "startup", ".io" | ".tech")
                | (_, ".com" | ".com.br")
        ),
        recommendation,
        status: if restricted {
            "restricted".into()
        } else {
            record_status.unwrap_or("available").into()
        },
        owner: record
            .filter(|_| record_status != Some("available"))
            .map(|item| item.owner.clone()),
        registered_at_year: record.map(|item| year(item.registered_at_seconds)),
        expires_at_year: record.map(|item| year(item.expires_at_seconds)),
    })
}

fn candidates(name: &str) -> Vec<String> {
    definitions()
        .into_iter()
        .filter(|item| item.suffix != ".gov.br")
        .map(|item| format!("{name}{}", item.suffix))
        .collect()
}

pub fn search(
    world: &WorldState,
    query: &str,
    business_type: &str,
) -> GameResult<DomainSearchResult> {
    if looks_like_onion(query) {
        return Err(domain(
            "Endereços .onion são serviços Tor v3 e não podem ser pesquisados, comprados ou registrados neste mercado.",
        ));
    }
    let name = normalized_name(query);
    if !valid_label(&name) {
        return Err(domain(
            "Informe um nome de domínio com letras, números ou hífens.",
        ));
    }
    let results = candidates(&name)
        .into_iter()
        .map(|address| offer(world, &address, business_type))
        .collect::<GameResult<Vec<_>>>()?;
    let suggestions = vec![
        format!("{name}.com.br"),
        format!("{name}.net"),
        format!("{name}.io"),
        format!("{name}app.com"),
        format!("use{name}.com"),
    ];
    Ok(DomainSearchResult {
        query: query.into(),
        normalized_name: name,
        results,
        suggestions,
    })
}

fn owner_record<'a>(world: &'a WorldState, address: &str) -> GameResult<&'a DomainRecord> {
    let parsed = parse(address)?;
    world
        .domains
        .registrations
        .get(&parsed.address)
        .filter(|record| {
            record.owner_kind == "player" && status(record, current(world)) != "available"
        })
        .ok_or_else(|| domain("Você não possui este domínio."))
}

fn owner_record_mut<'a>(
    world: &'a mut WorldState,
    address: &str,
) -> GameResult<&'a mut DomainRecord> {
    let parsed = parse(address)?;
    let now = current(world);
    world
        .domains
        .registrations
        .get_mut(&parsed.address)
        .filter(|record| record.owner_kind == "player" && status(record, now) != "available")
        .ok_or_else(|| domain("Você não possui este domínio."))
}

pub fn register(
    world: &mut WorldState,
    address: &str,
    organization: &str,
    business_type: &str,
) -> GameResult<DomainRecord> {
    let parsed = parse(address)?;
    if parsed.subdomain.is_some() {
        return Err(domain(
            "Registre o domínio principal antes de criar subdomínios.",
        ));
    }
    let extension = definition(&parsed.suffix)?;
    if parsed.government_namespace {
        return Err(domain(
            "Domínios governamentais são reservados a órgãos públicos.",
        ));
    }
    if let Some(reason) = restriction(&extension, business_type) {
        return Err(domain(reason));
    }
    let canonical = parsed.address.clone();
    let now = current(world);
    if world
        .domains
        .registrations
        .get(&canonical)
        .is_some_and(|record| status(record, now) != "available")
    {
        return Err(domain("Este domínio já está registrado."));
    }
    let organization = organization.trim();
    if organization.is_empty() || organization.len() > 80 {
        return Err(domain(
            "Informe o nome da empresa ou organização (até 80 caracteres).",
        ));
    }
    let purchase_price = price(&parsed.name, &extension);
    if world.money < purchase_price {
        return Err(domain(format!(
            "Saldo insuficiente. Este domínio custa ${purchase_price}."
        )));
    }
    let primary = !world.domains.registrations.values().any(|record| {
        record.owner_kind == "player" && record.primary && status(record, now) != "available"
    });
    world.money -= purchase_price;
    let record = DomainRecord {
        address: canonical.clone(),
        registered_name: parsed.name,
        suffix: parsed.suffix,
        category: extension.category,
        country: extension.country,
        owner: world.nickname.clone(),
        owner_kind: "player".into(),
        organization: organization.into(),
        business_type: business_type.into(),
        registered_at_seconds: now,
        expires_at_seconds: now + RENEWAL_CYCLE_SECONDS,
        auto_renew: true,
        primary,
        redirect_to: None,
        listed_price: None,
        subdomains: BTreeMap::new(),
    };
    world
        .domains
        .registrations
        .insert(canonical, record.clone());
    Ok(record)
}

pub fn renew(world: &mut WorldState, address: &str) -> GameResult<DomainRecord> {
    let now = current(world);
    let record = owner_record(world, address)?.clone();
    if now > record.expires_at_seconds + EXPIRY_GRACE_SECONDS {
        return Err(domain(
            "O período de renovação terminou; o domínio voltou ao mercado.",
        ));
    }
    let extension = definition(&record.suffix)?;
    if world.money < extension.renewal_price {
        return Err(domain("Saldo insuficiente para renovar este domínio."));
    }
    world.money -= extension.renewal_price;
    let updated = world
        .domains
        .registrations
        .get_mut(&record.address)
        .ok_or_else(|| domain("Domínio não encontrado."))?;
    updated.expires_at_seconds = updated.expires_at_seconds.max(now) + RENEWAL_CYCLE_SECONDS;
    updated.auto_renew = true;
    updated.listed_price = None;
    Ok(updated.clone())
}

pub fn set_primary(world: &mut WorldState, address: &str) -> GameResult<()> {
    let address = parse(address)?.address;
    owner_record(world, &address)?;
    let now = current(world);
    for record in world.domains.registrations.values_mut() {
        if record.owner_kind == "player" && status(record, now) != "available" {
            record.primary = record.address == address;
        }
    }
    Ok(())
}

pub fn create_subdomain(
    world: &mut WorldState,
    address: &str,
    label: &str,
) -> GameResult<SubdomainRecord> {
    let label = label.trim().to_lowercase();
    if !valid_label(&label) || label.contains('.') {
        return Err(domain(
            "Subdomínio inválido: use uma única palavra com letras, números ou hífens.",
        ));
    }
    if world.money < SUBDOMAIN_SETUP_COST {
        return Err(domain(
            "Saldo insuficiente para configurar este subdomínio.",
        ));
    }
    let record = owner_record(world, address)?.clone();
    if record.subdomains.contains_key(&label) {
        return Err(domain("Este subdomínio já existe."));
    }
    let subdomain = SubdomainRecord {
        label: label.clone(),
        address: format!("{label}.{}", record.address),
        created_at_seconds: current(world),
    };
    world.money -= SUBDOMAIN_SETUP_COST;
    owner_record_mut(world, &record.address)?
        .subdomains
        .insert(label, subdomain.clone());
    Ok(subdomain)
}

pub fn set_redirect(
    world: &mut WorldState,
    address: &str,
    target: Option<&str>,
) -> GameResult<DomainRecord> {
    let record = owner_record(world, address)?.clone();
    let target = target
        .map(|value| parse(value).map(|parsed| parsed.address))
        .transpose()?;
    if let Some(target) = &target {
        owner_record(world, target)?;
        if target == &record.address {
            return Err(domain("Um domínio não pode redirecionar para si mesmo."));
        }
    }
    let updated = owner_record_mut(world, &record.address)?;
    updated.redirect_to = target;
    Ok(updated.clone())
}

pub fn list_for_sale(
    world: &mut WorldState,
    address: &str,
    price: i64,
) -> GameResult<DomainMarketOffer> {
    if price <= 0 || price > 9_999_999 {
        return Err(domain("Informe um preço de venda entre $1 e $9.999.999."));
    }
    let record = owner_record(world, address)?.clone();
    let id = format!("{}:{}", record.address, current(world));
    let amount = price.saturating_mul(125).saturating_div(100).max(price + 1);
    let offer = DomainMarketOffer {
        id: id.clone(),
        address: record.address.clone(),
        buyer: "Nexora Ventures".into(),
        amount,
        status: "pending".into(),
        counter_price: None,
    };
    owner_record_mut(world, &record.address)?.listed_price = Some(price);
    world
        .domains
        .offers
        .retain(|item| item.address != record.address || item.status != "pending");
    world.domains.offers.push(offer.clone());
    Ok(offer)
}

pub fn cancel_sale(world: &mut WorldState, address: &str) -> GameResult<()> {
    let record = owner_record(world, address)?.clone();
    owner_record_mut(world, &record.address)?.listed_price = None;
    world
        .domains
        .offers
        .retain(|item| item.address != record.address || item.status != "pending");
    Ok(())
}

pub fn respond_offer(
    world: &mut WorldState,
    offer_id: &str,
    action: &str,
    counter_price: Option<i64>,
) -> GameResult<()> {
    let index = world
        .domains
        .offers
        .iter()
        .position(|offer| offer.id == offer_id && offer.status == "pending")
        .ok_or_else(|| domain("Oferta não encontrada ou já respondida."))?;
    let offer = world.domains.offers[index].clone();
    owner_record(world, &offer.address)?;
    match action {
        "accept" => {
            world.money = world.money.saturating_add(offer.amount);
            let record = owner_record_mut(world, &offer.address)?;
            record.owner = offer.buyer.clone();
            record.owner_kind = "npc".into();
            record.primary = false;
            record.listed_price = None;
            world.domains.offers[index].status = "accepted".into();
        }
        "reject" => {
            world.domains.offers[index].status = "rejected".into();
            owner_record_mut(world, &offer.address)?.listed_price = None;
        }
        "counter" => {
            let value = counter_price
                .filter(|value| *value > offer.amount && *value <= 9_999_999)
                .ok_or_else(|| domain("A contraproposta precisa ser maior que a oferta."))?;
            world.domains.offers[index].amount = value;
            world.domains.offers[index].counter_price = Some(value);
        }
        _ => return Err(domain("Resposta de oferta inválida.")),
    }
    Ok(())
}

pub fn whois(world: &WorldState, address: &str) -> GameResult<DomainWhois> {
    let normalized = normalize_address(address)?;
    if looks_like_onion(address) {
        let info = onion_service(world, address)?;
        return Ok(DomainWhois {
            address: info.address,
            ip: None,
            status: info.availability,
            owner: Some(info.owner),
            registered_at_year: None,
            expires_at_year: None,
            suffix: info.extension,
            category: "anonymous network".into(),
            country: None,
            subdomains: Vec::new(),
            redirect_to: None,
            network: Some(info.network),
            kind: Some(info.kind),
            uses_traditional_dns: Some(info.uses_traditional_dns),
            address_version: Some(info.address_version),
            registrable: Some(info.registrable),
            purchasable: Some(info.purchasable),
            tradable: Some(info.tradable),
        });
    }
    let parsed = parse(&normalized)?;
    let extension = definition(&parsed.suffix)?;
    let record = world.domains.registrations.get(&parsed.address);
    let state = record
        .map(|item| status(item, current(world)))
        .unwrap_or("available");
    let ip = world.network.resolve(&parsed.address).ok();
    Ok(DomainWhois {
        address: parsed.address,
        ip,
        status: state.into(),
        owner: record.map(|item| item.organization.clone()),
        registered_at_year: record.map(|item| year(item.registered_at_seconds)),
        expires_at_year: record.map(|item| year(item.expires_at_seconds)),
        suffix: extension.suffix,
        category: extension.category,
        country: extension.country,
        subdomains: record
            .map(|item| {
                item.subdomains
                    .values()
                    .map(|subdomain| subdomain.address.clone())
                    .collect()
            })
            .unwrap_or_default(),
        redirect_to: record.and_then(|item| item.redirect_to.clone()),
        network: Some("dns".into()),
        kind: Some("domain".into()),
        uses_traditional_dns: Some(true),
        address_version: None,
        registrable: Some(true),
        purchasable: Some(true),
        tradable: Some(true),
    })
}

impl DomainState {
    pub fn seeded() -> Self {
        let mut state = Self::empty();
        state.seed_npcs();
        state
    }

    fn empty() -> Self {
        Self {
            registrations: BTreeMap::new(),
            offers: Vec::new(),
            onion_services: seeded_onion_services(),
        }
    }

    fn seed_npcs(&mut self) {
        for (address, owner, organization, category) in [
            (
                "flash.com",
                "Flash Commerce",
                "Flash Commerce",
                "commercial",
            ),
            (
                "quantum.com",
                "Quantum Systems",
                "Quantum Systems",
                "technology",
            ),
            ("car.com", "Atlas Motors", "Atlas Motors", "commercial"),
            ("shop.com", "Shop Global", "Shop Global", "commercial"),
            ("bank.com", "Banco Meridian", "Banco Meridian", "commercial"),
            (
                "ai.com",
                "Axiom Intelligence",
                "Axiom Intelligence",
                "technology",
            ),
            ("food.com", "Food Planet", "Food Planet", "commercial"),
            (
                "getquantum.com",
                "Quantum Systems",
                "Quantum Systems",
                "technology",
            ),
            (
                "wipedia.org",
                "Wipedia Foundation",
                "Wipedia Foundation",
                "organization",
            ),
            (
                "archive.org",
                "Archive Foundation",
                "Archive Foundation",
                "organization",
            ),
            (
                "fakebook.com",
                "FakeBook Networks",
                "FakeBook Networks",
                "commercial",
            ),
            ("b1.tech", "B1 Media", "B1 Media", "technology"),
            (
                "mercado.com.br",
                "Mercado Aberto Ltda",
                "Mercado Aberto Ltda",
                "commercial",
            ),
            (
                "meudominio.com.br",
                "MeuDomínio Registro",
                "MeuDomínio Registro",
                "commercial",
            ),
            (
                "vigilia.org",
                "Sector IX Studio",
                "Sector IX Studio",
                "technology",
            ),
        ] {
            let parsed = parse(address).expect("valid seeded domain");
            self.registrations.insert(
                address.into(),
                DomainRecord {
                    address: address.into(),
                    registered_name: parsed.name,
                    suffix: parsed.suffix,
                    category: category.into(),
                    country: None,
                    owner: owner.into(),
                    owner_kind: "npc".into(),
                    organization: organization.into(),
                    business_type: category.into(),
                    registered_at_seconds: 0,
                    expires_at_seconds: u64::MAX / 2,
                    auto_renew: false,
                    primary: false,
                    redirect_to: None,
                    listed_price: None,
                    subdomains: BTreeMap::new(),
                },
            );
        }
    }

    pub fn validate(&self) -> GameResult<()> {
        for (key, record) in &self.registrations {
            if key != &record.address || parse(key).is_err() || record.owner.is_empty() {
                return Err(domain("Registro de domínio inválido."));
            }
            for (label, subdomain) in &record.subdomains {
                if label != &subdomain.label
                    || !valid_label(label)
                    || !subdomain
                        .address
                        .starts_with(&format!("{label}.{}", record.address))
                {
                    return Err(domain("Registro de subdomínio inválido."));
                }
            }
        }
        for (key, service) in &self.onion_services {
            if key != &service.address
                || !valid_onion_address(key)
                || service.service_name.is_empty()
                || !matches!(
                    service.availability_mode.as_str(),
                    "always" | "intermittent" | "offline"
                )
            {
                return Err(domain("Registro de Onion Service inválido."));
            }
        }
        Ok(())
    }
}

impl Default for DomainState {
    fn default() -> Self {
        Self::seeded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> WorldState {
        let mut world = WorldState::new("kali", "lifeos").unwrap();
        world.money = 20_000;
        world.domains = DomainState::seeded();
        world
    }

    #[test]
    fn parser_understands_suffixes_subdomains_and_government_hierarchy() {
        let parsed = parse("loja.empresa.com.br").unwrap();
        assert_eq!(parsed.name, "empresa");
        assert_eq!(parsed.suffix, ".com.br");
        assert_eq!(parsed.subdomain.as_deref(), Some("loja"));
        let government = parse("cidade.sp.gov.br").unwrap();
        assert_eq!(government.name, "cidade");
        assert_eq!(government.jurisdiction.as_deref(), Some("sp"));
        assert!(parse("empresa.org.sp.br").is_err());
    }

    #[test]
    fn search_prices_restrictions_premium_names_and_npc_availability() {
        let world = world();
        let result = search(&world, "Minha Empresa", "company").unwrap();
        assert_eq!(result.normalized_name, "minhaempresa");
        assert!(result
            .results
            .iter()
            .any(|item| item.address == "minhaempresa.com.br" && item.available));
        assert!(result
            .results
            .iter()
            .any(|item| item.address == "minhaempresa.com" && item.recommended));
        let premium = search(&world, "car", "company").unwrap();
        assert!(
            premium
                .results
                .iter()
                .find(|item| item.address == "car.com")
                .unwrap()
                .premium
        );
        assert!(
            !search(&world, "quantum", "company")
                .unwrap()
                .results
                .iter()
                .find(|item| item.address == "quantum.com")
                .unwrap()
                .available
        );
        assert!(
            search(&world, "instituto", "company")
                .unwrap()
                .results
                .iter()
                .find(|item| item.address == "instituto.org.br")
                .unwrap()
                .restricted
        );
        let archive_whois = whois(&world, "https://www.archive.org").unwrap();
        assert_eq!(archive_whois.ip.as_deref(), Some("10.20.4.20"));
        assert_eq!(archive_whois.owner.as_deref(), Some("Archive Foundation"));
        let onion = whois(&world, BLACKWIRE_ONION_ADDRESS).unwrap();
        assert_eq!(onion.ip, None);
        assert_eq!(onion.network.as_deref(), Some("tor"));
        assert_eq!(onion.address_version, Some(3));
        assert_eq!(onion.uses_traditional_dns, Some(false));
        assert!(!onion.registrable.unwrap());
        assert!(search(&world, BLACKWIRE_ONION_ADDRESS, "company").is_err());
    }

    #[test]
    fn onion_addresses_require_a_v3_cryptographic_label_and_track_availability_separately() {
        assert!(valid_onion_address(BLACKWIRE_ONION_ADDRESS));
        assert!(!valid_onion_address("abcdefghijklmnop.onion"));
        assert!(!valid_onion_address(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.onion"
        ));
        assert!(!valid_onion_address(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0.onion"
        ));
        assert!(!valid_onion_address(
            &BLACKWIRE_ONION_ADDRESS.to_ascii_uppercase()
        ));
        let mut world = world();
        let info = onion_service(&world, &format!("http://{BLACKWIRE_ONION_ADDRESS}")).unwrap();
        assert_eq!(info.network, "tor");
        assert_eq!(info.address_version, 3);
        assert_eq!(info.address_length, 56);
        assert!(!info.uses_traditional_dns);
        assert!(!info.purchasable);
        assert!(info.online);
        world.playtime_seconds = 3 * 24 * 60 * 60;
        let info = onion_service(&world, BLACKWIRE_ONION_ADDRESS).unwrap();
        assert!(!info.online);
        assert_eq!(info.availability, "offline");
        assert!(onion_online(&world, BLACKWIRE_ONION_ADDRESS).is_err());
    }

    #[test]
    fn purchase_renew_subdomain_primary_redirect_sale_and_whois_are_persistent_domain_actions() {
        let mut world = world();
        let record = register(&mut world, "cerqbus.com.br", "CerqBus", "company").unwrap();
        assert!(record.primary);
        let subdomain = create_subdomain(&mut world, &record.address, "app").unwrap();
        assert_eq!(subdomain.address, "app.cerqbus.com.br");
        register(&mut world, "cerqbus.io", "CerqBus", "startup").unwrap();
        set_primary(&mut world, "cerqbus.io").unwrap();
        set_redirect(&mut world, "cerqbus.com.br", Some("cerqbus.io")).unwrap();
        let offer = list_for_sale(&mut world, "cerqbus.com.br", 500).unwrap();
        assert_eq!(offer.amount, 625);
        respond_offer(&mut world, &offer.id, "counter", Some(700)).unwrap();
        respond_offer(&mut world, &offer.id, "accept", None).unwrap();
        assert_eq!(
            world.domains.registrations["cerqbus.com.br"].owner_kind,
            "npc"
        );
        let whois = whois(&world, "cerqbus.io").unwrap();
        assert_eq!(whois.owner.as_deref(), Some("CerqBus"));
    }

    #[test]
    fn restricted_domains_and_insufficient_funds_are_rejected_without_changes() {
        let mut world = world();
        let before = serde_json::to_string(&world).unwrap();
        assert!(register(&mut world, "loja.org.br", "Loja", "company").is_err());
        assert!(register(&mut world, "cidade.sp.gov.br", "Cidade", "company").is_err());
        world.money = 0;
        assert!(register(&mut world, "loja.com", "Loja", "company").is_err());
        world.money = 20_000;
        assert_eq!(
            world.domains.registrations.len(),
            DomainState::seeded().registrations.len()
        );
        assert!(!serde_json::to_string(&world).unwrap().contains("loja.com"));
        assert!(before.contains("quantum.com"));
    }
}
