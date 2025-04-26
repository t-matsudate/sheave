use std::{
    fs::remove_file,
    io::Result as IOResult,
    net::SocketAddr
};

#[cfg(feature = "sqlite")]
use sqlx::SqliteConnection as TopicStorage;
#[cfg(feature = "mysql")]
use sqlx::MySqlConnection as TopicStorage;
#[cfg(featue = "postgres")]
use sqlx::PgConnection as TopicStorage;

use sqlx::{
    Connection,
    query
};

use sheave_core::flv::Flv;
use super::stream_is_unpublished;

#[cfg(any(feature = "sqlite", feature = "postgres"))]
const STATEMENT: &str = "SELECT path FROM topics WHERE path = $1 AND client_addr = $2 AND unpublished_at ISNULL";
#[cfg(feature = "mysql")]
const STATEMENT: &str = "SELECT path FROM topics WHERE path = ? AND client_addr = ? AND unpublished_at IS NULL";


/// Checks whether specified topic got published.
///
/// # Panics
///
/// This becomes a panic unless the topic storage doesn't find.
pub async fn did_get_published(topic_storage_url: &str, topic_path: &str, client_addr: SocketAddr) -> bool {
    let mut connection = TopicStorage::connect(topic_storage_url).await.unwrap();

    query(STATEMENT)
        .bind(topic_path)
        .bind(client_addr.to_string())
        .fetch_one(&mut connection)
        .await
        .is_ok()
}

/// Creates specified file as a topic.
///
/// # Errors
///
/// * [`StreamIsUnpublished`]
///
/// When specified topic isn't in the database yet.
///
/// [`StreamIsUnpublished`]: super::StreamIsUnpublished
pub async fn publish_topic(topic_storage_url: &str, base_dir: &str, directory_separator: &str, topic_path: &str, client_addr: SocketAddr) -> IOResult<Flv> {
    if did_get_published(topic_storage_url, topic_path, client_addr).await {
        Flv::create(&format!("{base_dir}{directory_separator}{topic_path}.flv"))
    } else {
        Err(stream_is_unpublished(topic_path.into()))
    }
}

/// Removes specified file as a topic.
///
/// # Errors
///
/// * [`StreamIsUnpublished`]
///
/// When specified topic isn't in the database yet.
///
/// [`StreamIsUnpublished`]: super::StreamIsUnpublished
pub async fn unpublish_topic(topic_storage_url: &str, base_dir: &str, directory_separator: &str, topic_path: &str, client_addr: SocketAddr) -> IOResult<()> {
    if did_get_published(topic_storage_url, topic_path, client_addr).await {
        remove_file(format!("{base_dir}{directory_separator}{topic_path}.flv"))
    } else {
        Err(stream_is_unpublished(topic_path.into()))
    }
}

/// Opens specified file as a topic.
///
/// # Erorrs
///
/// * [`StreamIsUnpublished`]
///
/// When specified topic isn't in the database yet.
///
/// [`StreamIsUnpublished`]: super::StreamIsUnpublished
pub async fn subscribe_topic(topic_storage_url: &str, base_dir: &str, directory_separator: &str, topic_path: &str, client_addr: SocketAddr) -> IOResult<Flv> {
    if did_get_published(topic_storage_url, topic_path, client_addr).await {
        Flv::open(&format!("{base_dir}{directory_separator}{topic_path}.flv"))
    } else {
        Err(stream_is_unpublished(topic_path.into()))
    }
}
