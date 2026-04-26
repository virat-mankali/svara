use anyhow::Context;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionEntry {
    pub id: String,
    pub text: String,
    pub source: String,
    pub created_at: String,
}

pub struct HistoryDb {
    conn: Connection,
}

impl HistoryDb {
    pub fn new() -> anyhow::Result<Self> {
        let path = config::app_dir()?.join("history.db");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)
            .with_context(|| format!("failed to open {}", path.display()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                source TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn insert_entry(&self, text: &str, source: &str) -> anyhow::Result<TranscriptionEntry> {
        let entry = TranscriptionEntry {
            id: Uuid::new_v4().to_string(),
            text: text.to_string(),
            source: source.to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        self.conn.execute(
            "INSERT INTO transcriptions (id, text, source, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![entry.id, entry.text, entry.source, entry.created_at],
        )?;

        self.conn.execute(
            "DELETE FROM transcriptions
             WHERE id NOT IN (
                SELECT id FROM transcriptions ORDER BY created_at DESC LIMIT 100
             )",
            [],
        )?;

        Ok(entry)
    }

    pub fn get_all_entries(&self) -> anyhow::Result<Vec<TranscriptionEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, text, source, created_at
             FROM transcriptions
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TranscriptionEntry {
                id: row.get(0)?,
                text: row.get(1)?,
                source: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn delete_entry(&self, id: &str) -> anyhow::Result<()> {
        self.conn
            .execute("DELETE FROM transcriptions WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_all(&self) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM transcriptions", [])?;
        Ok(())
    }
}
