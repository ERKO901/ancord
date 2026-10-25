//! Vážený rendezvous hash: anchory mailboxu v epoše (`spec/PROTOCOL.md` §7).

/// Váha uzlu pro mailbox v epoše: `blake3("ancord/v1/hrw" ‖ 0x00 ‖ mailbox ‖ node_id ‖ 0x00 ‖ epoch_le_u64)`.
pub fn weight(mailbox_id: &[u8; 32], node_id: &str, epoch: u64) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"ancord/v1/hrw");
    h.update(&[0]);
    h.update(mailbox_id);
    h.update(node_id.as_bytes());
    h.update(&[0]);
    h.update(&epoch.to_le_bytes());
    *h.finalize().as_bytes()
}

/// `r` nodů s nejvyšší váhou (big-endian porovnání), při shodě vyhrává menší `node_id`.
pub fn anchors<'a>(mailbox_id: &[u8; 32], node_ids: impl IntoIterator<Item = &'a str>, epoch: u64, r: usize) -> Vec<String> {
    let mut scored: Vec<([u8; 32], &str)> = node_ids.into_iter().map(|id| (weight(mailbox_id, id, epoch), id)).collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
    scored.into_iter().take(r).map(|(_, id)| id.to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_bounded() {
        let mailbox = [7u8; 32];
        let ids = ["c", "a", "b", "d"];
        let first = anchors(&mailbox, ids, 5, 3);
        assert_eq!(first.len(), 3);
        assert_eq!(first, anchors(&mailbox, ["d", "b", "a", "c"], 5, 3), "order of input must not matter");
        assert_eq!(anchors(&mailbox, ids, 5, 10).len(), 4);
    }
}
