use std::{path::Path, sync::Arc, time::Duration};

use moka::future::Cache;
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::models::{friend::Friend, users::User, world::World};

use super::{AppSnapshot, RecentPipelineEvent, SessionMetadata};

const USERS: TableDefinition<&str, &[u8]> = TableDefinition::new("users");
const FRIENDS: TableDefinition<&str, &[u8]> = TableDefinition::new("friends");
const WORLDS: TableDefinition<&str, &[u8]> = TableDefinition::new("worlds");
const NOTIFICATIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("notifications");
const METADATA: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");
const EVENTS: TableDefinition<u64, &[u8]> = TableDefinition::new("recent_events");
const SESSION_KEY: &str = "session";

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("local database error: {0}")]
    Database(String),
    #[error("local cache serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_users: u64,
    pub max_friends: u64,
    pub max_worlds: u64,
    pub max_notifications: u64,
    pub max_recent_events: usize,
    pub time_to_live: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_users: 2_000,
            max_friends: 2_000,
            max_worlds: 1_000,
            max_notifications: 500,
            max_recent_events: 200,
            time_to_live: Duration::from_secs(30 * 60),
        }
    }
}

#[derive(Clone)]
pub struct LocalCache {
    database: Arc<Database>,
    users: Cache<String, Arc<User>>,
    friends: Cache<String, Arc<Friend>>,
    worlds: Cache<String, Arc<World>>,
    notifications: Cache<String, Arc<Value>>,
    pub(crate) max_recent_events: usize,
}

impl LocalCache {
    pub fn open(path: impl AsRef<Path>, config: CacheConfig) -> Result<Self, StoreError> {
        let database =
            Database::create(path).map_err(|error| StoreError::Database(error.to_string()))?;

        let write = database
            .begin_write()
            .map_err(|error| StoreError::Database(error.to_string()))?;
        write
            .open_table(USERS)
            .map_err(|error| StoreError::Database(error.to_string()))?;
        write
            .open_table(FRIENDS)
            .map_err(|error| StoreError::Database(error.to_string()))?;
        write
            .open_table(WORLDS)
            .map_err(|error| StoreError::Database(error.to_string()))?;
        write
            .open_table(NOTIFICATIONS)
            .map_err(|error| StoreError::Database(error.to_string()))?;
        write
            .open_table(METADATA)
            .map_err(|error| StoreError::Database(error.to_string()))?;
        write
            .open_table(EVENTS)
            .map_err(|error| StoreError::Database(error.to_string()))?;
        write
            .commit()
            .map_err(|error| StoreError::Database(error.to_string()))?;

        Ok(Self {
            database: Arc::new(database),
            users: build_cache(config.max_users, config.time_to_live),
            friends: build_cache(config.max_friends, config.time_to_live),
            worlds: build_cache(config.max_worlds, config.time_to_live),
            notifications: build_cache(config.max_notifications, config.time_to_live),
            max_recent_events: config.max_recent_events,
        })
    }

    pub async fn put_user(&self, user: User) -> Result<(), StoreError> {
        let id = user.identity.id.clone();
        put_json(&self.database, USERS, &id, &user)?;
        self.users.insert(id, Arc::new(user)).await;
        Ok(())
    }

    pub async fn user(&self, id: &str) -> Result<Option<Arc<User>>, StoreError> {
        get_cached(&self.database, USERS, &self.users, id).await
    }

    pub async fn put_friend(&self, friend: Friend) -> Result<(), StoreError> {
        let id = friend.id.clone();
        put_json(&self.database, FRIENDS, &id, &friend)?;
        self.friends.insert(id, Arc::new(friend)).await;
        Ok(())
    }

    pub async fn friend(&self, id: &str) -> Result<Option<Arc<Friend>>, StoreError> {
        get_cached(&self.database, FRIENDS, &self.friends, id).await
    }

    pub async fn remove_friend(&self, id: &str) -> Result<(), StoreError> {
        remove(&self.database, FRIENDS, id)?;
        self.friends.invalidate(id).await;
        Ok(())
    }

    pub async fn put_world(&self, world: World) -> Result<(), StoreError> {
        let id = world.identifier.id.clone();
        put_json(&self.database, WORLDS, &id, &world)?;
        self.worlds.insert(id, Arc::new(world)).await;
        Ok(())
    }

    pub async fn world(&self, id: &str) -> Result<Option<Arc<World>>, StoreError> {
        get_cached(&self.database, WORLDS, &self.worlds, id).await
    }

    pub async fn put_notification(&self, id: &str, value: Value) -> Result<(), StoreError> {
        put_json(&self.database, NOTIFICATIONS, id, &value)?;
        self.notifications
            .insert(id.to_string(), Arc::new(value))
            .await;
        Ok(())
    }

    pub async fn notification(&self, id: &str) -> Result<Option<Arc<Value>>, StoreError> {
        get_cached(&self.database, NOTIFICATIONS, &self.notifications, id).await
    }

    pub async fn remove_notification(&self, id: &str) -> Result<(), StoreError> {
        remove(&self.database, NOTIFICATIONS, id)?;
        self.notifications.invalidate(id).await;
        Ok(())
    }

    pub fn put_session_metadata(&self, metadata: &SessionMetadata) -> Result<(), StoreError> {
        put_json(&self.database, METADATA, SESSION_KEY, metadata)
    }

    pub(crate) fn append_event(&self, event: &RecentPipelineEvent) -> Result<(), StoreError> {
        let write = self
            .database
            .begin_write()
            .map_err(|error| StoreError::Database(error.to_string()))?;
        {
            let mut table = write
                .open_table(EVENTS)
                .map_err(|error| StoreError::Database(error.to_string()))?;
            table
                .insert(event.sequence, serde_json::to_vec(event)?.as_slice())
                .map_err(|error| StoreError::Database(error.to_string()))?;

            while table
                .len()
                .map_err(|error| StoreError::Database(error.to_string()))?
                > self.max_recent_events as u64
            {
                let first = table
                    .first()
                    .map_err(|error| StoreError::Database(error.to_string()))?
                    .map(|(key, _)| key.value());
                if let Some(key) = first {
                    table
                        .remove(key)
                        .map_err(|error| StoreError::Database(error.to_string()))?;
                }
            }
        }
        write
            .commit()
            .map_err(|error| StoreError::Database(error.to_string()))
    }

    pub(crate) fn load_snapshot(&self) -> Result<AppSnapshot, StoreError> {
        let session = get_json(&self.database, METADATA, SESSION_KEY)?.unwrap_or_default();
        let recent_events = read_all_values(&self.database, EVENTS)?.into();
        let notifications = read_string_map(&self.database, NOTIFICATIONS)?;

        Ok(AppSnapshot {
            session,
            notifications,
            recent_events,
            ..AppSnapshot::default()
        })
    }
}

fn build_cache<T: Send + Sync + 'static>(capacity: u64, ttl: Duration) -> Cache<String, Arc<T>> {
    Cache::builder()
        .max_capacity(capacity)
        .time_to_live(ttl)
        .build()
}

async fn get_cached<T>(
    database: &Database,
    table: TableDefinition<'static, &str, &[u8]>,
    cache: &Cache<String, Arc<T>>,
    id: &str,
) -> Result<Option<Arc<T>>, StoreError>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    if let Some(value) = cache.get(id).await {
        return Ok(Some(value));
    }
    let value = get_json(database, table, id)?.map(Arc::<T>::new);
    if let Some(value) = &value {
        cache.insert(id.to_string(), value.clone()).await;
    }
    Ok(value)
}

fn put_json<K, T>(
    database: &Database,
    definition: TableDefinition<K, &[u8]>,
    key: K::SelfType<'_>,
    value: &T,
) -> Result<(), StoreError>
where
    K: redb::Key + 'static,
    T: Serialize,
{
    let bytes = serde_json::to_vec(value)?;
    let write = database
        .begin_write()
        .map_err(|error| StoreError::Database(error.to_string()))?;
    {
        let mut table = write
            .open_table(definition)
            .map_err(|error| StoreError::Database(error.to_string()))?;
        table
            .insert(key, bytes.as_slice())
            .map_err(|error| StoreError::Database(error.to_string()))?;
    }
    write
        .commit()
        .map_err(|error| StoreError::Database(error.to_string()))
}

fn get_json<K, T>(
    database: &Database,
    definition: TableDefinition<K, &[u8]>,
    key: K::SelfType<'_>,
) -> Result<Option<T>, StoreError>
where
    K: redb::Key + 'static,
    T: DeserializeOwned,
{
    let read = database
        .begin_read()
        .map_err(|error| StoreError::Database(error.to_string()))?;
    let table = read
        .open_table(definition)
        .map_err(|error| StoreError::Database(error.to_string()))?;
    table
        .get(key)
        .map_err(|error| StoreError::Database(error.to_string()))?
        .map(|value| serde_json::from_slice(value.value()))
        .transpose()
        .map_err(StoreError::from)
}

fn remove(
    database: &Database,
    definition: TableDefinition<&str, &[u8]>,
    key: &str,
) -> Result<(), StoreError> {
    let write = database
        .begin_write()
        .map_err(|error| StoreError::Database(error.to_string()))?;
    {
        let mut table = write
            .open_table(definition)
            .map_err(|error| StoreError::Database(error.to_string()))?;
        table
            .remove(key)
            .map_err(|error| StoreError::Database(error.to_string()))?;
    }
    write
        .commit()
        .map_err(|error| StoreError::Database(error.to_string()))
}

fn read_all_values<T>(
    database: &Database,
    definition: TableDefinition<u64, &[u8]>,
) -> Result<Vec<T>, StoreError>
where
    T: DeserializeOwned,
{
    let read = database
        .begin_read()
        .map_err(|error| StoreError::Database(error.to_string()))?;
    let table = read
        .open_table(definition)
        .map_err(|error| StoreError::Database(error.to_string()))?;
    table
        .iter()
        .map_err(|error| StoreError::Database(error.to_string()))?
        .map(|entry| {
            let (_, value) = entry.map_err(|error| StoreError::Database(error.to_string()))?;
            serde_json::from_slice(value.value()).map_err(StoreError::from)
        })
        .collect()
}

fn read_string_map(
    database: &Database,
    definition: TableDefinition<&str, &[u8]>,
) -> Result<std::collections::HashMap<String, Value>, StoreError> {
    let read = database
        .begin_read()
        .map_err(|error| StoreError::Database(error.to_string()))?;
    let table = read
        .open_table(definition)
        .map_err(|error| StoreError::Database(error.to_string()))?;
    table
        .iter()
        .map_err(|error| StoreError::Database(error.to_string()))?
        .map(|entry| {
            let (key, value) = entry.map_err(|error| StoreError::Database(error.to_string()))?;
            Ok((
                key.value().to_string(),
                serde_json::from_slice(value.value())?,
            ))
        })
        .collect()
}
