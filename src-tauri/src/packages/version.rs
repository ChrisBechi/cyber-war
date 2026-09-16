//! Debian ordering, including epoch, revision, tilde and numeric runs.
use std::cmp::Ordering;

fn split(value: &str) -> (&str, &str, &str) {
    let (epoch, rest) = value.split_once(':').unwrap_or(("0", value));
    let (upstream, revision) = rest.rsplit_once('-').unwrap_or((rest, "0"));
    (epoch, upstream, revision)
}
fn number(a: &str, b: &str) -> Ordering {
    let a = a.trim_start_matches('0');
    let b = b.trim_start_matches('0');
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}
fn order(c: Option<u8>) -> i32 {
    match c {
        Some(b'~') => -1,
        None => 0,
        Some(c) if c.is_ascii_digit() => 0,
        Some(c) if c.is_ascii_alphabetic() => i32::from(c),
        Some(c) => i32::from(c) + 256,
    }
}
fn part(a: &str, b: &str) -> Ordering {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        while a.get(i).is_some_and(|c| !c.is_ascii_digit())
            || b.get(j).is_some_and(|c| !c.is_ascii_digit())
        {
            let cmp = order(a.get(i).copied()).cmp(&order(b.get(j).copied()));
            if cmp != Ordering::Equal {
                return cmp;
            }
            if i < a.len() {
                i += 1;
            }
            if j < b.len() {
                j += 1;
            }
        }
        while a.get(i) == Some(&b'0') {
            i += 1;
        }
        while b.get(j) == Some(&b'0') {
            j += 1;
        }
        let (start_i, start_j) = (i, j);
        while a.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        while b.get(j).is_some_and(u8::is_ascii_digit) {
            j += 1;
        }
        let cmp = (i - start_i)
            .cmp(&(j - start_j))
            .then_with(|| a[start_i..i].cmp(&b[start_j..j]));
        if cmp != Ordering::Equal {
            return cmp;
        }
    }
    Ordering::Equal
}
pub fn compare(a: &str, b: &str) -> Ordering {
    let (ae, au, ar) = split(a);
    let (be, bu, br) = split(b);
    number(ae, be)
        .then_with(|| part(au, bu))
        .then_with(|| part(ar, br))
}
pub fn valid(value: &str) -> bool {
    if value.is_empty() || value.len() > 128 || !value.is_ascii() {
        return false;
    }
    let (e, u, r) = split(value);
    !e.is_empty()
        && e.bytes().all(|c| c.is_ascii_digit())
        && u.as_bytes().first().is_some_and(u8::is_ascii_digit)
        && u.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".+-:~".contains(&c))
        && !r.is_empty()
        && r.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"+.~".contains(&c))
}
pub fn matches(version: &str, op: &str, required: &str) -> bool {
    let cmp = compare(version, required);
    match op {
        "" => true,
        "=" => cmp == Ordering::Equal,
        ">=" => cmp != Ordering::Less,
        "<=" => cmp != Ordering::Greater,
        ">>" => cmp == Ordering::Greater,
        "<<" => cmp == Ordering::Less,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn debian_ordering() {
        for (a, b) in [
            ("1.0~rc1", "1.0"),
            ("1.9", "1.10"),
            ("1.0-2", "1.0-10"),
            ("9.9", "1:1.0"),
            ("1.0a", "1.0+"),
        ] {
            assert_eq!(compare(a, b), Ordering::Less, "{a} < {b}");
            assert_eq!(compare(b, a), Ordering::Greater);
        }
        assert_eq!(compare("1.01", "1.1"), Ordering::Equal);
        assert_eq!(compare("1.0", "1.0-0"), Ordering::Equal);
        for value in ["", "virtual", "1-", "x:1", "1/2"] {
            assert!(!valid(value));
        }
    }
}
