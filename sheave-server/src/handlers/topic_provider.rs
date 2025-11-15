use std::{
    fs::remove_file,
    io::Result as IOResult,
    net::SocketAddr,
    path::PathBuf,
};

#[cfg(feature = "sqlite")]
use sqlx::SqliteConnection as Connector;
#[cfg(feature = "mysql")]
use sqlx::MySqlConnection as Connector;
#[cfg(featue = "postgres")]
use sqlx::PgConnection as Connector;

use sqlx::{
    Connection,
    query
};

use sheave_core::flv::Flv;
use super::stream_is_unpublished;

#[cfg(any(feature = "sqlite", feature = "postgres"))]
const STATEMENT: &str = "SELECT id FROM topics WHERE id = $1 AND client_addr = $2 AND unpublished_at ISNULL";
#[cfg(feature = "mysql")]
const STATEMENT: &str = "SELECT id FROM topics WHERE id = ? AND client_addr = ? AND unpublished_at IS NULL";

/// Checks whether specified topic got published.
///
/// # Panics
///
/// This becomes a panic unless the topic database doesn't find.
pub async fn did_get_published(database_url: &str, topic_id: &str, client_addr: SocketAddr) -> bool {
    let mut connection = Connector::connect(database_url).await.unwrap();

    query(STATEMENT)
        .bind(topic_id)
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
pub async fn publish_topic(database_url: &str, storage_path: &str, app: &str, topic_id: &str, client_addr: SocketAddr) -> IOResult<Flv> {
    if did_get_published(database_url, topic_id, client_addr).await {
        let mut topic_path = PathBuf::from(storage_path);
        topic_path.push(app);
        topic_path.push(format!("{topic_id}.flv"));
        Flv::create(topic_path)
    } else {
        Err(stream_is_unpublished(topic_id.into()))
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
pub async fn unpublish_topic(database_url: &str, storage_path: &str, app: &str, topic_id: &str, client_addr: SocketAddr) -> IOResult<()> {
    if did_get_published(database_url, topic_id, client_addr).await {
        let mut topic_path = PathBuf::from(storage_path);
        topic_path.push(app);
        topic_path.push(format!("{topic_id}.flv"));
        remove_file(topic_path)
    } else {
        Err(stream_is_unpublished(topic_id.into()))
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
pub async fn subscribe_topic(database_url: &str, storage_path: &str, app: &str, topic_id: &str, client_addr: SocketAddr) -> IOResult<Flv> {
    if did_get_published(database_url, topic_id, client_addr).await {
        let mut topic_path = PathBuf::from(storage_path);
        topic_path.push(app);
        topic_path.push(format!("{topic_id}.flv"));
        Flv::open(topic_path)
    } else {
        Err(stream_is_unpublished(topic_id.into()))
    }
}
