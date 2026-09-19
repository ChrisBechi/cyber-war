//! Popularity follows authored engagement, without invented random counts.
use super::model::{Detail, Document};
pub fn score(doc: &Document) -> u32 {
    let audience = match &doc.detail {
        Detail::Video { views, .. } => u64::from(*views),
        Detail::SocialPost { likes, .. } => u64::from(*likes) * 20,
        Detail::Thread { votes, .. } => u64::from(*votes) * 10,
        Detail::SocialProfile { following, .. } => following.len() as u64 * 5,
        _ => 0,
    };
    let evidence = audience.saturating_add(doc.comments.len() as u64 * 25);
    // Logarithmic bands keep a large channel from overwhelming topical relevance.
    let engagement = if evidence == 0 {
        0
    } else {
        evidence.ilog2().min(20) * 3
    };
    (35 + engagement).min(95)
}

#[cfg(test)]
mod tests {
    #[test]
    fn engagement_is_monotonic_bounded_and_never_random() {
        let mut doc = super::super::repository()
            .pack
            .documents
            .iter()
            .find(|d| matches!(d.detail, super::Detail::SocialPost { .. }))
            .unwrap()
            .clone();
        let mut previous = 0;
        for value in [0, 1, 10, 100, 1000, u32::MAX] {
            if let super::Detail::SocialPost { likes, .. } = &mut doc.detail {
                *likes = value;
            }
            let score = super::score(&doc);
            assert!(score >= previous && score <= 95);
            assert_eq!(score, super::score(&doc.clone()));
            previous = score;
        }
    }
}
