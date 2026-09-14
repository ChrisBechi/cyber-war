use crate::{error::GameResult, vfs::domain, world::WorldState};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Partition {
    pub size: f64,
    pub fs: String,
    pub mount: String,
}

fn setting<'a>(world: &'a WorldState, key: &str, fallback: &'a str) -> &'a str {
    world
        .settings
        .get(key)
        .map(String::as_str)
        .unwrap_or(fallback)
}

pub fn validate(world: &WorldState, json: &str) -> GameResult<Vec<Partition>> {
    let parts: Vec<Partition> =
        serde_json::from_str(json).map_err(|_| domain("invalid partition table"))?;
    let capacity = if setting(world, "storage", "plain") == "raid1" {
        64.2
    } else if setting(world, "partition", "guided-disk") == "guided-largest" {
        if setting(world, "disk", "nvme0n1") == "sda" {
            32.0
        } else {
            200.0
        }
    } else if setting(world, "disk", "nvme0n1") == "sda" {
        64.2
    } else {
        500.1
    };
    let linux = ["ext4", "ext3", "ext2", "btrfs", "xfs"];
    let valid = !parts.is_empty()
        && parts.len() <= 64
        && parts.iter().all(|p| {
            p.size.is_finite()
                && p.size > 0.0
                && (linux.contains(&p.fs.as_str())
                    || ["FAT32", "FAT16", "ESP", "swap"].contains(&p.fs.as_str()))
                && ((p.fs == "swap" && p.mount == "swap")
                    || (p.mount.starts_with('/')
                        && !p.mount.chars().any(char::is_whitespace)
                        && !p.mount.contains("..")))
        })
        && parts
            .iter()
            .any(|p| p.mount == "/" && p.size >= 7.0 && linux.contains(&p.fs.as_str()))
        && parts
            .iter()
            .any(|p| p.fs == "ESP" && p.mount == "/boot/efi" && p.size >= 0.1)
        && parts.iter().map(|p| p.size).sum::<f64>() <= capacity + 0.001;
    let mut mounts = std::collections::BTreeSet::new();
    if !valid
        || parts
            .iter()
            .any(|p| p.mount != "swap" && !mounts.insert(&p.mount))
    {
        return Err(domain(
            "invalid partition layout or insufficient disk space",
        ));
    }
    Ok(parts)
}

fn device(world: &WorldState, part: &Partition, index: usize) -> String {
    let disk = setting(world, "disk", "nvme0n1");
    let storage = setting(world, "storage", "plain");
    if part.fs != "ESP" {
        match storage {
            "lvm" | "encrypted" => return format!("/dev/mapper/kali--vg-lv{}", index + 1),
            "raid1" => return format!("/dev/md0p{}", index + 1),
            "iscsi" => return format!("/dev/sdb{}", index + 1),
            _ => {}
        }
    }
    let offset = usize::from(setting(world, "partition", "guided-disk") == "guided-largest");
    format!(
        "/dev/{disk}{}{}",
        if disk == "sda" { "" } else { "p" },
        index + 1 + offset
    )
}

pub fn apply(world: &mut WorldState, json: &str) -> GameResult<()> {
    let parts = validate(world, json)?;
    let mut fstab = String::from("# Kali Linux - tabela de particoes do sistema virtual\n");
    for (index, p) in parts.iter().enumerate() {
        let fs = match p.fs.as_str() {
            "ESP" | "FAT32" | "FAT16" => "vfat",
            other => other,
        };
        fstab.push_str(&format!(
            "{} {} {} defaults 0 {}\n",
            device(world, p, index),
            p.mount,
            fs,
            if p.mount == "/" { 1 } else { 0 }
        ));
        if p.mount.starts_with('/') && !world.vfs.nodes.contains_key(&p.mount) {
            world.vfs.seed(&p.mount, "directory", "", "root");
        }
    }
    world.vfs.seed("/etc/fstab", "file", &fstab, "root");
    world.vfs.seed(
        "/etc/kali-storage",
        "file",
        &format!("{}\n", setting(world, "storage", "plain")),
        "root",
    );
    Ok(())
}

pub fn lsblk(world: &WorldState) -> Option<String> {
    let parts = validate(world, world.settings.get("partitions")?).ok()?;
    let disk = setting(world, "disk", "nvme0n1");
    let storage = setting(world, "storage", "plain");
    let mut result = format!(
        "NAME SIZE TYPE FSTYPE MOUNTPOINT\n{disk} {}G disk\n",
        if disk == "sda" { 64.2 } else { 500.1 }
    );
    if setting(world, "partition", "guided-disk") == "guided-largest" {
        result.push_str(&format!(
            "{disk}{}1 {}G part ntfs /dados\n",
            if disk == "sda" { "" } else { "p" },
            if disk == "sda" { 32.2 } else { 300.1 }
        ));
    }
    if storage != "plain" {
        result.push_str(&format!("{storage} volume\n"));
    }
    for (index, p) in parts.iter().enumerate() {
        result.push_str(&format!(
            "{} {:.1}G {} {} {}\n",
            device(world, p, index).trim_start_matches("/dev/"),
            p.size,
            if storage == "plain" || p.fs == "ESP" {
                "part"
            } else {
                "lvm"
            },
            p.fs,
            p.mount
        ));
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    const PLAN: &str = r#"[{"size":1,"fs":"ESP","mount":"/boot/efi"},{"size":20,"fs":"ext4","mount":"/"},{"size":15,"fs":"btrfs","mount":"/home"},{"size":4,"fs":"swap","mount":"swap"}]"#;
    #[test]
    fn applies_custom_usb_layout_to_virtual_fstab_and_lsblk() {
        let mut world = WorldState::new("kali", "kali").unwrap();
        world.settings.insert("disk".into(), "sda".into());
        world.settings.insert("partition".into(), "manual".into());
        apply(&mut world, PLAN).unwrap();
        world.settings.insert("partitions".into(), PLAN.into());
        assert!(world.vfs.nodes["/etc/fstab"]
            .content
            .contains("/dev/sda3 /home btrfs"));
        assert!(lsblk(&world)
            .unwrap()
            .contains("sda3 15.0G part btrfs /home"));
        assert!(validate(&world, &PLAN.replace("\"size\":20", "\"size\":200")).is_err());
        assert!(validate(&world, &PLAN.replace("/home", "/")).is_err());
    }
    #[test]
    fn lvm_and_preserved_space_are_reflected_in_device_names() {
        let mut world = WorldState::new("kali", "kali").unwrap();
        world
            .settings
            .insert("partition".into(), "guided-largest".into());
        world.settings.insert("partitions".into(), PLAN.into());
        assert!(lsblk(&world).unwrap().contains("ntfs /dados"));
        world.settings.insert("storage".into(), "encrypted".into());
        assert!(lsblk(&world).unwrap().contains("mapper/kali--vg-lv2"));
    }
}
