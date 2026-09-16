use super::*;
use std::{
    collections::BTreeMap,
    io::{self, Cursor, Read, Write},
};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

const MANIFEST: &str = ".cyber-war-vfs-v1.json";
pub(crate) fn failure(e: impl std::fmt::Display) -> crate::error::GameError {
    domain(format!("archive: {e}"))
}

pub(crate) fn read_bounded(reader: impl Read, limit: u64) -> GameResult<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut reader = reader.take(limit + 1);
    let mut buffer = [0u8; 16384];
    loop {
        jobs::check_cancel()?;
        let read = reader.read(&mut buffer).map_err(failure)?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    if bytes.len() as u64 > limit {
        return Err(domain("archive: decompression size limit exceeded"));
    }
    Ok(bytes)
}
pub(crate) fn decoder<'a>(
    bytes: &'a [u8],
    format: ArchiveFormat,
) -> GameResult<Box<dyn Read + 'a>> {
    Ok(match format.stream() {
        ArchiveFormat::Gzip => Box::new(flate2::read::MultiGzDecoder::new(bytes)),
        ArchiveFormat::Bzip2 => Box::new(bzip2::read::MultiBzDecoder::new(bytes)),
        ArchiveFormat::Xz => {
            let stream = liblzma::stream::Stream::new_stream_decoder(
                64 * 1024 * 1024,
                liblzma::stream::CONCATENATED,
            )
            .map_err(failure)?;
            Box::new(liblzma::read::XzDecoder::new_stream(bytes, stream))
        }
        _ => Box::new(Cursor::new(bytes)),
    })
}
pub(crate) fn compress(bytes: &[u8], format: ArchiveFormat) -> GameResult<Vec<u8>> {
    match format.stream() {
        ArchiveFormat::Gzip => {
            let mut w = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            w.write_all(bytes).map_err(failure)?;
            w.finish().map_err(failure)
        }
        ArchiveFormat::Bzip2 => {
            let mut w = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::default());
            w.write_all(bytes).map_err(failure)?;
            w.finish().map_err(failure)
        }
        ArchiveFormat::Xz => {
            let mut w = liblzma::write::XzEncoder::new(Vec::new(), 3);
            w.write_all(bytes).map_err(failure)?;
            w.finish().map_err(failure)
        }
        ArchiveFormat::Tar => Ok(bytes.to_vec()),
        _ => Err(domain("archive: expected a compression format")),
    }
}

pub(crate) fn encode(
    entries: &[StoredEntry],
    format: ArchiveFormat,
    password: Option<&str>,
    limits: &ArchiveSafetyLimits,
) -> GameResult<Vec<u8>> {
    limits.entries(entries, limits.max_storage_size)?;
    if entries.iter().any(|e| e.info.path == MANIFEST) {
        return Err(domain("archive: reserved metadata filename"));
    }
    if password.is_some() && format != ArchiveFormat::Zip {
        return Err(domain("archive: passwords require ZIP"));
    }
    let metadata: BTreeMap<_, _> = entries
        .iter()
        .filter_map(|e| e.metadata.as_ref().map(|m| (&e.info.path, m)))
        .collect();
    let manifest = serde_json::to_vec(&metadata)?;
    let meta = StoredEntry {
        info: ArchiveEntry {
            path: MANIFEST.into(),
            kind: "file".into(),
            original_size: manifest.len() as u64,
            compressed_size: None,
            permissions: 0o600,
            modified_at: 0,
            owner: None,
            group: None,
            crc: None,
            link_target: None,
            encrypted: false,
        },
        data: manifest,
        metadata: None,
    };
    let result = if format == ArchiveFormat::Zip {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for e in std::iter::once(&meta).chain(entries.iter()) {
            let mut opts = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(e.info.permissions as u32);
            if let Some(password) = password {
                opts = opts.with_aes_encryption(zip::AesMode::Aes256, password);
            }
            match e.info.kind.as_str() {
                "directory" => writer
                    .add_directory(format!("{}/", e.info.path), opts)
                    .map_err(failure)?,
                "symlink" => {
                    writer
                        .add_symlink(
                            &e.info.path,
                            e.info.link_target.as_deref().unwrap_or(""),
                            opts,
                        )
                        .map_err(failure)?;
                }
                _ => {
                    writer.start_file(&e.info.path, opts).map_err(failure)?;
                    writer.write_all(&e.data).map_err(failure)?;
                }
            }
        }
        writer.finish().map_err(failure)?.into_inner()
    } else {
        if format.is_stream() {
            return Err(domain("archive: use compressFile for single streams"));
        }
        let mut builder = tar::Builder::new(Vec::new());
        for e in std::iter::once(&meta).chain(entries.iter()) {
            let mut h = tar::Header::new_gnu();
            h.set_mode(e.info.permissions as u32);
            h.set_mtime(e.info.modified_at);
            h.set_uid(0);
            h.set_gid(0);
            h.set_username(e.info.owner.as_deref().unwrap_or("kali"))
                .map_err(failure)?;
            h.set_groupname(e.info.group.as_deref().unwrap_or("kali"))
                .map_err(failure)?;
            h.set_entry_type(match e.info.kind.as_str() {
                "directory" => tar::EntryType::Directory,
                "symlink" => tar::EntryType::Symlink,
                _ => tar::EntryType::Regular,
            });
            h.set_size(e.data.len() as u64);
            if let Some(link) = &e.info.link_target {
                h.set_link_name(link).map_err(failure)?;
            }
            h.set_cksum();
            builder
                .append_data(&mut h, &e.info.path, e.data.as_slice())
                .map_err(failure)?;
        }
        let data = builder.into_inner().map_err(failure)?;
        if data.len() as u64 > limits.max_storage_size {
            return Err(domain("archive: storage size limit exceeded"));
        }
        compress(&data, format)?
    };
    if result.len() as u64 > limits.max_storage_size {
        return Err(domain("archive: storage size limit exceeded"));
    }
    Ok(result)
}

fn zip_count(bytes: &[u8], limits: &ArchiveSafetyLimits) -> GameResult<usize> {
    // Preflight before the ZIP crate allocates the directory. ZIP64 is deliberately rejected.
    let pos = (bytes.len().saturating_sub(65557)..bytes.len().saturating_sub(21))
        .rev()
        .find(|p| {
            bytes.get(*p..*p + 4) == Some(b"PK\x05\x06")
                && *p + 22 + u16::from_le_bytes([bytes[*p + 20], bytes[*p + 21]]) as usize
                    == bytes.len()
        })
        .ok_or_else(|| domain("End-of-central-directory signature not found."))?;
    let count = u16::from_le_bytes([bytes[pos + 10], bytes[pos + 11]]) as usize;
    if count > limits.max_entries + 1 || bytes[pos + 4..pos + 8] != [0, 0, 0, 0] {
        return Err(domain(
            "archive: entry count limit or unsupported split/ZIP64 archive",
        ));
    }
    Ok(count)
}

pub(crate) fn decode(
    bytes: &[u8],
    format: ArchiveFormat,
    password: Option<&str>,
    materialize: bool,
    limits: &ArchiveSafetyLimits,
) -> GameResult<Vec<StoredEntry>> {
    let mut entries = Vec::new();
    let mut metadata: BTreeMap<String, EntryMetadata> = BTreeMap::new();
    let mut declared = 0u64;
    let mut manifest_seen = false;
    if format == ArchiveFormat::Zip {
        let count = zip_count(bytes, limits)?;
        let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(failure)?;
        if zip.len() != count {
            return Err(domain("archive: duplicate filename or invalid directory"));
        }
        for i in 0..zip.len() {
            let raw = zip.by_index_raw(i).map_err(failure)?;
            let p = limits.path(raw.name())?;
            let mode = raw.unix_mode().unwrap_or(0o644);
            let kind = if raw.is_dir() {
                "directory"
            } else if mode & 0o170000 == 0o120000 {
                "symlink"
            } else {
                "file"
            };
            let mut info = ArchiveEntry {
                path: p,
                kind: kind.into(),
                original_size: raw.size(),
                compressed_size: Some(raw.compressed_size()),
                permissions: (mode & 0o777) as u16,
                modified_at: 0,
                owner: None,
                group: None,
                crc: Some(raw.crc32()),
                link_target: None,
                encrypted: raw.encrypted(),
            };
            declared = declared
                .checked_add(raw.size())
                .ok_or_else(|| domain("archive: size overflow"))?;
            if declared > limits.max_storage_size
                || raw.size()
                    > raw
                        .compressed_size()
                        .max(1)
                        .saturating_mul(limits.max_compression_ratio)
            {
                return Err(domain("archive: decompression size/ratio limit exceeded"));
            }
            drop(raw);
            let need_data = materialize
                || (info.kind == "symlink" && (!info.encrypted || password.is_some()))
                || (info.path == MANIFEST && (!info.encrypted || password.is_some()));
            let data = if need_data {
                let reader = if let Some(pw) = password {
                    zip.by_index_decrypt(i, pw.as_bytes())
                } else {
                    zip.by_index(i)
                }
                .map_err(|e| {
                    if info.encrypted {
                        domain(if password.is_some() {
                            "incorrect password"
                        } else {
                            "archive: password required"
                        })
                    } else {
                        failure(e)
                    }
                })?;
                read_bounded(
                    reader,
                    if info.path == MANIFEST {
                        1024 * 1024
                    } else {
                        limits.max_storage_size
                    },
                )?
            } else {
                Vec::new()
            };
            if info.path == MANIFEST {
                if !data.is_empty() {
                    metadata = serde_json::from_slice(&data).map_err(failure)?;
                }
                continue;
            }
            if info.kind == "symlink" && need_data {
                info.link_target = Some(String::from_utf8(data.clone()).map_err(failure)?);
            }
            entries.push(StoredEntry {
                info,
                data,
                metadata: None,
            });
        }
    } else {
        let reader = decoder(bytes, format)?.take(limits.max_storage_size + 1);
        let mut archive = tar::Archive::new(reader);
        let mut consumed = 0u64;
        let mut root_seen = false;
        for (index, entry) in archive.entries().map_err(failure)?.enumerate() {
            let mut entry = entry.map_err(failure)?;
            if index > limits.max_entries {
                return Err(domain("archive: entry count limit exceeded"));
            }
            let path_bytes = entry.path_bytes();
            let raw_path = std::str::from_utf8(&path_bytes).map_err(failure)?;
            let raw_path = raw_path.trim_start_matches("./");
            if raw_path.is_empty() && entry.header().entry_type().is_dir() && entry.size() == 0 {
                if root_seen {
                    return Err(domain("archive: duplicate root directory"));
                }
                root_seen = true;
                consumed += 512;
                continue;
            }
            let p = limits.path(raw_path)?;
            let h = entry.header();
            let kind = if h.entry_type().is_dir() {
                "directory"
            } else if h.entry_type().is_symlink() {
                "symlink"
            } else if h.entry_type().is_file() {
                "file"
            } else {
                return Err(domain("archive: unsupported TAR entry type"));
            };
            declared = declared
                .checked_add(entry.size())
                .ok_or_else(|| domain("archive: size overflow"))?;
            if declared > limits.max_storage_size
                || declared
                    > (bytes.len() as u64)
                        .max(1)
                        .saturating_mul(limits.max_compression_ratio)
            {
                return Err(domain("archive: decompression size/ratio limit exceeded"));
            }
            let info = ArchiveEntry {
                path: p,
                kind: kind.into(),
                original_size: entry.size(),
                compressed_size: None,
                permissions: (h.mode().map_err(failure)? & 0o777) as u16,
                modified_at: h.mtime().map_err(failure)?,
                owner: h.username().map_err(failure)?.map(String::from),
                group: h.groupname().map_err(failure)?.map(String::from),
                crc: None,
                link_target: entry
                    .link_name_bytes()
                    .map(|p| String::from_utf8(p.to_vec()))
                    .transpose()
                    .map_err(failure)?,
                encrypted: false,
            };
            let data = if materialize || info.path == MANIFEST {
                read_bounded(
                    &mut entry,
                    if info.path == MANIFEST {
                        1024 * 1024
                    } else {
                        limits.max_storage_size
                    },
                )?
            } else {
                io::copy(&mut entry, &mut io::sink()).map_err(failure)?;
                Vec::new()
            };
            consumed += 512 + info.original_size.div_ceil(512) * 512;
            if info.path == MANIFEST {
                if manifest_seen {
                    return Err(domain("archive: duplicate metadata filename"));
                }
                manifest_seen = true;
                metadata = serde_json::from_slice(&data).map_err(failure)?;
            } else {
                entries.push(StoredEntry {
                    info,
                    data,
                    metadata: None,
                });
            }
        }
        let mut reader = archive.into_inner();
        io::copy(&mut reader, &mut io::sink()).map_err(failure)?;
        if reader.limit() == 0 {
            return Err(domain("archive: decompression size limit exceeded"));
        }
        let actual = limits.max_storage_size + 1 - reader.limit();
        if actual < consumed + 1024 {
            return Err(domain("tar: truncated archive"));
        }
    }
    for entry in &mut entries {
        if let Some(m) = metadata.remove(&entry.info.path) {
            if m.logical_size < entry.info.original_size || m.archive_depth > 8 {
                return Err(domain("archive: invalid virtual metadata"));
            }
            entry.info.original_size = m.logical_size;
            entry.info.modified_at = m.modified_at;
            entry.info.owner = Some(m.owner.clone());
            entry.info.group = Some(m.group.clone());
            entry.metadata = Some(m);
        }
    }
    limits.entries(&entries, bytes.len() as u64)?;
    Ok(entries)
}
