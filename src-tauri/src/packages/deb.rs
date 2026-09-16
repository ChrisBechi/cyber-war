//! Bounded ar envelope; TAR and compression remain owned by ArchiveService.
use super::{model::*, version};
use crate::{
    archive::{self, ArchiveEntry, ArchiveFormat, ArchiveSafetyLimits, StoredEntry},
    error::GameResult,
    vfs::domain,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const MIME: &str = "application/vnd.debian.binary-package";
const MAGIC: &[u8] = b"!<arch>\n";
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn detected(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}
pub fn valid_name(name: &str) -> bool {
    name.len() >= 2
        && name.len() <= 128
        && name
            .bytes()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"+.-".contains(&c))
}
pub fn validate(p: &Definition) -> GameResult<()> {
    if !valid_name(&p.name)
        || !version::valid(&p.version)
        || !["amd64", "all"].contains(&p.architecture.as_str())
    {
        return Err(domain("dpkg: invalid metadata or unsupported architecture"));
    }
    if p.description.is_empty()
        || p.description.len() > 16384
        || p.download_size > 16 * 1024 * 1024 * 1024
    {
        return Err(domain("dpkg: invalid package size or description"));
    }
    let limits = ArchiveSafetyLimits::default();
    if p.actions.len() > 128
        || p.depends.len() > 128
        || p.recommends.len() > 128
        || p.suggests.len() > 128
        || p.depends
            .iter()
            .chain(&p.recommends)
            .chain(&p.suggests)
            .any(|c| c.len() > 32)
        || p.conflicts.len() > 128
        || p.provides.len() > 128
        || p.replaces.len() > 128
    {
        return Err(domain("dpkg: metadata complexity limit"));
    }
    let mut paths = BTreeSet::new();
    let mut logical = 0u64;
    let mut physical = 0u64;
    if p.files.len() > limits.max_entries {
        return Err(domain("dpkg: payload entry limit"));
    }
    for f in &p.files {
        let relative = f
            .path
            .strip_prefix('/')
            .ok_or_else(|| domain("dpkg: absolute virtual path required"))?;
        limits.path(relative)?;
        if !["/usr/", "/etc/", "/opt/", "/var/lib/"]
            .iter()
            .any(|prefix| f.path.starts_with(prefix))
            || ["/var/lib/dpkg", "/var/lib/apt", "/etc/apt"]
                .iter()
                .any(|root| f.path == *root || f.path.starts_with(&format!("{root}/")))
            || !paths.insert(f.path.clone())
            || f.mode > 0o777
            || !["file", "directory", "symlink"].contains(&f.kind.as_str())
            || f.logical_size < f.content.len() as u64
            || (f.kind == "directory" && (!f.content.is_empty() || f.logical_size != 0))
        {
            return Err(domain("dpkg: invalid or duplicate payload path/metadata"));
        }
        if f.conffile && (f.kind != "file" || !f.path.starts_with("/etc/")) {
            return Err(domain("dpkg: invalid conffile"));
        }
        if f.kind == "symlink" {
            limits.path(&f.content)?;
        }
        if let Some(binding) = &f.binding {
            if f.kind != "file"
                || f.mode & 0o111 == 0
                || !f.path.starts_with("/usr/bin/")
                || !(binding == "netscan"
                    || binding == "wireless"
                    || binding == "iot"
                    || binding == "builtin"
                    || binding.starts_with("catalog:")
                        && crate::software::by_command(&binding[8..]).is_some())
            {
                return Err(domain("dpkg: unsupported executable binding"));
            }
        }
        logical = logical
            .checked_add(f.logical_size)
            .ok_or_else(|| domain("dpkg: logical size overflow"))?;
        physical = physical
            .checked_add(f.content.len() as u64)
            .ok_or_else(|| domain("dpkg: storage overflow"))?;
    }
    if logical > limits.max_uncompressed_size || physical > limits.max_storage_size {
        return Err(domain("dpkg: payload size limit"));
    }
    for f in &p.files {
        if p.files.iter().any(|parent| {
            parent.kind != "directory" && f.path.starts_with(&format!("{}/", parent.path))
        }) {
            return Err(domain("dpkg: payload traverses file or symlink"));
        }
    }
    for relation in p
        .depends
        .iter()
        .chain(&p.recommends)
        .chain(&p.suggests)
        .flatten()
        .chain(&p.conflicts)
        .chain(&p.provides)
        .chain(&p.replaces)
    {
        if !valid_name(&relation.name)
            || !["", "=", "<<", "<=", ">=", ">>"].contains(&relation.op.as_str())
            || (!relation.op.is_empty() && !version::valid(&relation.version))
        {
            return Err(domain("dpkg: invalid dependency relation"));
        }
    }
    if p.depends
        .iter()
        .chain(&p.recommends)
        .chain(&p.suggests)
        .any(Vec::is_empty)
    {
        return Err(domain("dpkg: empty dependency clause"));
    }
    for action in &p.actions {
        if let Action::DefaultFile { content, .. } = action {
            if content.len() > 1024 * 1024 {
                return Err(domain("dpkg: maintainer file too large"));
            }
        }
        let path = match action {
            Action::EnsureDirectory { path } | Action::DefaultFile { path, .. } => path,
        };
        if !path.starts_with(&format!("/var/lib/{}/", p.name)) {
            return Err(domain(
                "dpkg: maintainer action outside package data directory",
            ));
        }
        limits.path(&path[1..])?;
    }
    Ok(())
}
fn entry(path: &str, kind: &str, data: Vec<u8>, mode: u16, target: Option<String>) -> StoredEntry {
    StoredEntry {
        info: ArchiveEntry {
            path: path.into(),
            kind: kind.into(),
            original_size: data.len() as u64,
            compressed_size: None,
            permissions: mode,
            modified_at: 0,
            owner: Some("root".into()),
            group: Some("root".into()),
            crc: None,
            link_target: target,
            encrypted: false,
        },
        data,
        metadata: None,
    }
}
fn member(out: &mut Vec<u8>, name: &str, bytes: &[u8]) {
    out.extend_from_slice(
        format!(
            "{:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
            format!("{name}/"),
            0,
            0,
            0,
            "100644",
            bytes.len()
        )
        .as_bytes(),
    );
    out.extend_from_slice(bytes);
    if !bytes.len().is_multiple_of(2) {
        out.push(b'\n');
    }
}
pub fn control(p: &Definition) -> String {
    let clauses = |relations: &[Vec<Relation>]| {
        relations
            .iter()
            .map(|clause| {
                clause
                    .iter()
                    .map(|r| {
                        if r.op.is_empty() {
                            r.name.clone()
                        } else {
                            format!("{} ({} {})", r.name, r.op, r.version)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" | ")
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let relations =
        |items: &[Relation]| clauses(&items.iter().cloned().map(|r| vec![r]).collect::<Vec<_>>());
    format!("Package: {}\nVersion: {}\nArchitecture: {}\nPriority: {}\nSection: {}\nMaintainer: {}\nInstalled-Size: {}\nDepends: {}\nRecommends: {}\nSuggests: {}\nConflicts: {}\nProvides: {}\nReplaces: {}\nEssential: {}\nHomepage: {}\nDownload-Size: {}\nDescription: {}\n", p.name,p.version,p.architecture,p.priority,p.section,p.maintainer,p.installed_size().div_ceil(1024),clauses(&p.depends),clauses(&p.recommends),clauses(&p.suggests),relations(&p.conflicts),relations(&p.provides),relations(&p.replaces),if p.essential {"yes"} else {"no"},p.homepage,p.download_size,p.description.replace('\n',"\n "))
}
fn conffiles(p: &Definition) -> String {
    p.files
        .iter()
        .filter(|f| f.conffile)
        .map(|f| format!("{}\n", f.path))
        .collect()
}
pub fn encode(p: &Definition) -> GameResult<Vec<u8>> {
    validate(p)?;
    let mut controls = vec![
        entry("control", "file", control(p).into_bytes(), 0o644, None),
        entry(
            "cyber-war.json",
            "file",
            serde_json::to_vec(p)?,
            0o644,
            None,
        ),
    ];
    if p.files.iter().any(|f| f.conffile) {
        controls.push(entry(
            "conffiles",
            "file",
            conffiles(p).into_bytes(),
            0o644,
            None,
        ));
    }
    let payload = p
        .files
        .iter()
        .map(|f| {
            entry(
                &f.path[1..],
                &f.kind,
                if f.kind == "file" {
                    f.content.as_bytes().to_vec()
                } else {
                    Vec::new()
                },
                f.mode,
                (f.kind == "symlink").then(|| f.content.clone()),
            )
        })
        .collect::<Vec<_>>();
    let mut out = MAGIC.to_vec();
    member(&mut out, "debian-binary", b"2.0\n");
    member(
        &mut out,
        "control.tar.gz",
        &archive::encode_entries(&controls, ArchiveFormat::TarGzip)?,
    );
    member(
        &mut out,
        "data.tar.xz",
        &archive::encode_entries(&payload, ArchiveFormat::TarXz)?,
    );
    Ok(out)
}
pub fn decode(bytes: &[u8]) -> GameResult<Definition> {
    if !detected(bytes) || bytes.len() > 32 * 1024 * 1024 {
        return Err(domain("dpkg: invalid Debian package header or size"));
    }
    let mut offset = MAGIC.len();
    let mut members = Vec::new();
    while offset < bytes.len() {
        let header = bytes
            .get(offset..offset + 60)
            .ok_or_else(|| domain("dpkg: truncated ar header"))?;
        if &header[58..] != b"`\n" {
            return Err(domain("dpkg: corrupt ar member"));
        }
        let name = std::str::from_utf8(&header[..16])
            .map_err(|_| domain("dpkg: invalid ar name"))?
            .trim()
            .trim_end_matches('/');
        let size = std::str::from_utf8(&header[48..58])
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .ok_or_else(|| domain("dpkg: invalid ar size"))?;
        offset += 60;
        let end = offset
            .checked_add(size)
            .ok_or_else(|| domain("dpkg: ar size overflow"))?;
        let data = bytes
            .get(offset..end)
            .ok_or_else(|| domain("dpkg: truncated ar member"))?;
        members.push((name, data));
        offset = end
            .checked_add(size % 2)
            .ok_or_else(|| domain("dpkg: ar size overflow"))?;
        if offset > bytes.len() || members.len() > 3 {
            return Err(domain("dpkg: invalid ar members"));
        }
    }
    if members.len() != 3 || members[0] != ("debian-binary", b"2.0\n".as_slice()) {
        return Err(domain("dpkg: unsupported Debian package layout"));
    }
    let format = |name: &str, prefix: &str| -> GameResult<ArchiveFormat> {
        match name.strip_prefix(prefix) {
            Some("") => Ok(ArchiveFormat::Tar),
            Some(".gz") => Ok(ArchiveFormat::TarGzip),
            Some(".xz") => Ok(ArchiveFormat::TarXz),
            Some(".bz2") if prefix == "data.tar" => Ok(ArchiveFormat::TarBzip2),
            _ => Err(domain("dpkg: unsupported inner archive")),
        }
    };
    let control_entries =
        archive::decode_entries(members[1].1, format(members[1].0, "control.tar")?)?;
    if control_entries
        .iter()
        .any(|e| !["control", "cyber-war.json", "conffiles"].contains(&e.info.path.as_str()))
    {
        return Err(domain(
            "dpkg: unsupported maintainer script or control member",
        ));
    }
    let manifest = control_entries
        .iter()
        .find(|e| e.info.path == "cyber-war.json")
        .ok_or_else(|| domain("dpkg: package is not a supported virtual profile"))?;
    let p: Definition = serde_json::from_slice(&manifest.data)?;
    validate(&p)?;
    if control_entries.iter().any(|e| e.info.kind != "file")
        || control_entries
            .iter()
            .find(|e| e.info.path == "conffiles")
            .is_some_and(|e| e.data != conffiles(&p).as_bytes())
    {
        return Err(domain("dpkg: invalid control entries/conffiles"));
    }
    if !control_entries
        .iter()
        .any(|e| e.info.path == "control" && e.data == control(&p).as_bytes())
    {
        return Err(domain("dpkg: control metadata mismatch"));
    }
    let files = archive::decode_entries(members[2].1, format(members[2].0, "data.tar")?)?;
    if files.len() != p.files.len() {
        return Err(domain("dpkg: payload manifest mismatch"));
    }
    for f in &p.files {
        let e = files
            .iter()
            .find(|e| e.info.path == f.path[1..])
            .ok_or_else(|| domain("dpkg: payload file missing"))?;
        if e.info.kind != f.kind
            || e.info.permissions != f.mode
            || (f.kind == "file" && e.data != f.content.as_bytes())
            || (f.kind == "symlink" && e.info.link_target.as_deref() != Some(f.content.as_str()))
        {
            return Err(domain("dpkg: payload checksum mismatch"));
        }
    }
    Ok(p)
}
