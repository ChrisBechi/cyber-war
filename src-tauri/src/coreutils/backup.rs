//! Shared backup naming policy over the virtual directory namespace.
use super::foundation_options::quote;
use crate::{terminal_io::Output, vfs::VirtualFileSystem};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Strategy {
    None,
    Simple,
    Existing,
    Numbered,
}
impl Strategy {
    pub fn parse(value: &str, source: &str, invocation: &str) -> Result<Self, Box<Output>> {
        let aliases = [
            ("none", Self::None),
            ("off", Self::None),
            ("simple", Self::Simple),
            ("never", Self::Simple),
            ("existing", Self::Existing),
            ("nil", Self::Existing),
            ("numbered", Self::Numbered),
            ("t", Self::Numbered),
        ];
        if value.is_empty() {
            return Ok(Self::Existing);
        }
        if let Some((_, mode)) = aliases.iter().find(|(n, _)| *n == value) {
            return Ok(*mode);
        }
        let matches: Vec<_> = aliases
            .iter()
            .filter(|(n, _)| n.starts_with(value))
            .collect();
        if matches.len() == 1 {
            return Ok(matches[0].1);
        }
        Err(Box::new(Output { status:1, stderr:format!("ln: {} argument {} for '{}'\nValid arguments are:\n  - 'none', 'off'\n  - 'simple', 'never'\n  - 'existing', 'nil'\n  - 'numbered', 't'\nTry '{invocation} --help' for more information.\n",if matches.is_empty() {"invalid"} else {"ambiguous"},quote(value),source), ..Default::default() }))
    }
    pub fn name(self, fs: &VirtualFileSystem, path: &str, actor: &str, suffix: &str) -> String {
        let (dir, base) = path.rsplit_once('/').unwrap_or((".", path));
        let dir = if dir.is_empty() { "/" } else { dir };
        let prefix = format!("{base}.~");
        let names = fs.child_names(dir, actor).unwrap_or_default();
        let max = names
            .iter()
            .filter_map(|n| n.strip_prefix(&prefix)?.strip_suffix('~'))
            .filter(|s| {
                s.as_bytes()
                    .first()
                    .is_some_and(|b| (b'1'..=b'9').contains(b))
                    && s.bytes().all(|b| b.is_ascii_digit())
            })
            .max_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        let numbered = self == Self::Numbered || (self == Self::Existing && max.is_some());
        let extension = if numbered {
            let mut next = max.unwrap_or("0").as_bytes().to_vec();
            let mut carry = true;
            for b in next.iter_mut().rev() {
                if *b == b'9' {
                    *b = b'0';
                } else {
                    *b += 1;
                    carry = false;
                    break;
                }
            }
            if carry {
                next.insert(0, b'1');
            }
            format!(
                ".~{}~",
                String::from_utf8(next).expect("decimal backup version")
            )
        } else {
            suffix.into()
        };
        let candidate = format!("{path}{extension}");
        if self != Self::Simple && base.len() + extension.len() > 255 {
            // The locked reference's fallback reserves space for its terminator
            // and replaces an overlong extension with one tilde.
            let mut end = base.len().min(253);
            while !base.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}/{}~", dir.trim_end_matches('/'), &base[..end])
        } else {
            candidate
        }
    }
}
