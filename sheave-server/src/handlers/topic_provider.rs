use std::{
    io::{
        Error as IOError,
        Result as IOResult
    },
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
    query,
    query_as
};

use sheave_core::flv::Flv;

/// Checks whether specified topic got published.
///
/// # Panics
///
/// This becomes a panic unless the topic storage doesn't find.
pub async fn did_get_published(topic_storage_url: &str, playpath: &str, client_addr: SocketAddr) -> bool {
    let mut connection = TopicStorage::connect(topic_storage_url).await.unwrap();

    #[cfg(any(feature = "sqlite", feature = "postgres"))]
    let statement = "FROM topics SELECT playpath WHERE playpath = $1 AND client_addr = $2";
    #[cfg(feature = "mysql")]
    let statement = "FROM topics SELECT playpath WHERE playpath = ? AND client_addr = ?";

    query(statement)
        .bind(playpath)
        .bind(client_addr.to_string())
        .fetch_one(&mut connection)
        .await.is_ok()
}

/// Registers specified topic requested from a server.
///
/// # Panics
///
/// This becomes a panic unless the topic storage doesn't find.
pub async fn publish_topic(topic_storage_url: &str, playpath: &str, client_addr: SocketAddr) -> IOResult<()> {
    let mut connection = TopicStorage::connect(topic_storage_url).await.unwrap();

    #[cfg(any(feature = "sqlite", feature = "postgres"))]
    let statement = "INSERT INTO topics VALUES ($1, $2)";
    #[cfg(feature = "mysql")]
    let statement = "INSERT INTO topics VALUES (?, ?)";

    query(statement)
        .bind(playpath)
        .bind(client_addr.to_string())
        .execute(&mut connection)
        .await
        .map(|_| ())
        .map_err(IOError::other)
}

/// Unregisters specified topic requested from a server.
///
/// # Panics
///
/// This becomes a panic unless the topic storage doesn't find.
pub async fn unpublish_topic(topic_storage_url: &str, playpath: &str, client_addr: SocketAddr) -> IOResult<()> {
    let mut connection = TopicStorage::connect(topic_storage_url).await.unwrap();

    #[cfg(any(feature = "sqlite", feature = "postgres"))]
    let statement = "UPDATE topics SET unpublished_at=CURRENT_TIMESTAMP WHERE playpath=$1 AND client_addr=$2";
    #[cfg(feature = "mysql")]
    let statement = "UPDATE topics SET unpublished_at=CURRENT_TIMESTAMP WHERE playpath=? AND client_addr=?";

    query(statement)
        .bind(playpath)
        .bind(client_addr.to_string())
        .execute(&mut connection)
        .await
        .map(|_| ())
        .map_err(IOError::other)
}

/// Distributes a topic requested from a server.
///
/// # Panics
///
/// This becomes a topic unless the topic storage doesn't find.
pub async fn subscribe_topic<'r>(topic_storage_url: &str, playpath: &str) -> IOResult<Flv> {
    let mut connection = TopicStorage::connect(topic_storage_url).await.unwrap();

    #[cfg(any(feature = "sqlite", feature = "postgres"))]
    let statement = "FROM topics SELECT playpath WHERE playpath=$1";
    #[cfg(feature = "mysql")]
    let statement = "FROM topics SELECT playpath WHERE playpath=?";

    let (playpath,): (String,) = query_as(statement)
        .bind(playpath)
        .fetch_one(&mut connection)
        .await
        .map_err(IOError::other)?;
    Flv::open(&playpath)
}
