use slotmap::DefaultKey;

/// Arena slot key — the internal storage index in the `SlotMap`.
///
/// `ObjectId` (u64) is the stable public identity; `SlotKey` is the O(1)
/// arena accessor used only inside `ObjectTree`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SlotKey(pub(crate) DefaultKey);

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::SlotMap;

    #[test]
    fn slot_key_copy_and_eq() {
        let mut sm: SlotMap<DefaultKey, u32> = SlotMap::new();
        let k = SlotKey(sm.insert(42));
        let k2 = k;
        assert_eq!(k, k2);
    }
}
