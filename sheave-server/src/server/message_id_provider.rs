use std::sync::{
    Mutex,
    atomic::{
        AtomicU32,
        Ordering
    }
};

static CURRENT_MAX_ID: AtomicU32 = AtomicU32::new(u32::MIN);
static VACANT_IDS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

/// Provides a message ID to a server instance.
/// This considers following things to provide it:
///
/// 1. Checks whether some vacant ID exists. Because of avoiding exhausted ID ranges.
/// 2. If it exists, this provides a new ID, then increases current max number.
/// 3. Otherwise reuses previous vacancies.
pub async fn provide_message_id() -> u32 {
    let mut vacant_ids = VACANT_IDS.lock().unwrap();

    if vacant_ids.is_empty() {
        CURRENT_MAX_ID.fetch_add(1, Ordering::Relaxed)
    } else {
        vacant_ids.pop().unwrap()
    }
}

/// Registers a message ID released from a server.
/// This considers following things to register it:
///
/// 1. Checks whether it is equal to current max number. Because of avoiding to duplicate IDs which are provided.
/// 2. If it is, decreases current max number, then registers it.
/// 3. Otherwise keeps current max number as it is.
pub async fn return_message_id(message_id: u32) {
    let mut vacant_ids = VACANT_IDS.lock().unwrap();

    if message_id == CURRENT_MAX_ID.load(Ordering::Relaxed) {
        CURRENT_MAX_ID.fetch_sub(1, Ordering::Relaxed);
    }

    vacant_ids.push(message_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn reset_max_id() {
        CURRENT_MAX_ID.store(u32::default(), Ordering::Relaxed);
    }

    async fn reset_vacant_ids() {
        let mut vacant_ids = VACANT_IDS.lock().unwrap();
        *vacant_ids = Vec::default();
    }

    #[tokio::test]
    async fn provide_when_nothing_vacant() {
        reset_max_id().await;
        reset_vacant_ids().await;
        let message_id = provide_message_id().await;
        assert_eq!(0, message_id);
        assert_eq!(1, CURRENT_MAX_ID.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn provide_from_vacant_ids() {
        reset_max_id().await;
        reset_vacant_ids().await;
        let previous_message_id = provide_message_id().await;
        // Emits one more message ID for expressing a vacant state.
        let _ = provide_message_id().await;
        return_message_id(previous_message_id).await;
        let next_message_id = provide_message_id().await;
        assert_eq!(previous_message_id, next_message_id);
        assert_eq!(2, CURRENT_MAX_ID.load(Ordering::Relaxed))
    }
}
