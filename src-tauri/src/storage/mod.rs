use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub permissions_enabled: bool,
    pub voice_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<serde_json::Value>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub category: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct Storage {
    db_path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_local_dir()
            .context("Failed to get local data directory")?;
        let app_dir = data_dir.join("E.D.I.T.H");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&app_dir)
            .context("Failed to create app directory")?;

        let db_path = app_dir.join("edith.db");

        let storage = Self { db_path };
        storage.initialize_db()?;
        Ok(storage)
    }

    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
            .context("Failed to open database connection")
    }

    fn initialize_db(&self) -> Result<()> {
        let conn = self.get_connection()?;

        // Create settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        ).context("Failed to create settings table")?;

        // Create conversations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                messages TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        ).context("Failed to create conversations table")?;

        // Create memory table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memory (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        ).context("Failed to create memory table")?;

        // Create action log table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS action_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                action TEXT NOT NULL,
                tool TEXT NOT NULL,
                result TEXT NOT NULL
            )",
            [],
        ).context("Failed to create action log table")?;

        Ok(())
    }

    // Settings operations
    pub fn save_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        ).context("Failed to save setting")?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get(0)).ok();
        Ok(result)
    }

    pub fn delete_setting(&self, key: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])
            .context("Failed to delete setting")?;
        Ok(())
    }

    // Conversation operations
    pub fn save_conversation(&self, conversation: &Conversation) -> Result<()> {
        let conn = self.get_connection()?;
        let messages_json = serde_json::to_string(&conversation.messages)
            .context("Failed to serialize conversation messages")?;

        conn.execute(
            "INSERT OR REPLACE INTO conversations (id, title, messages, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                conversation.id,
                conversation.title,
                messages_json,
                conversation.created_at,
                conversation.updated_at
            ],
        ).context("Failed to save conversation")?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_conversation(&self, id: &str) -> Result<Option<Conversation>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, messages, created_at, updated_at FROM conversations WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            let messages_json: String = row.get(2)?;
            let messages = serde_json::from_str(&messages_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                messages,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        }).ok();

        Ok(result)
    }

    pub fn list_conversations(&self) -> Result<Vec<Conversation>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, messages, created_at, updated_at FROM conversations ORDER BY updated_at DESC"
        )?;

        let conversations = stmt.query_map([], |row| {
            let messages_json: String = row.get(2)?;
            let messages = serde_json::from_str(&messages_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                messages,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()
            .context("Failed to collect conversations")?;

        Ok(conversations)
    }

    pub fn delete_conversation(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])
            .context("Failed to delete conversation")?;
        Ok(())
    }

    // Memory operations
    pub fn save_memory(&self, memory: &MemoryEntry) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT OR REPLACE INTO memory (id, category, content, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                memory.id,
                memory.category,
                memory.content,
                memory.created_at,
                memory.updated_at
            ],
        ).context("Failed to save memory")?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_memory(&self, id: &str) -> Result<Option<MemoryEntry>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, category, content, created_at, updated_at FROM memory WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(MemoryEntry {
                id: row.get(0)?,
                category: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        }).ok();

        Ok(result)
    }

    pub fn list_memory(&self) -> Result<Vec<MemoryEntry>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, category, content, created_at, updated_at FROM memory ORDER BY updated_at DESC"
        )?;

        let memory_entries = stmt.query_map([], |row| {
            Ok(MemoryEntry {
                id: row.get(0)?,
                category: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()
            .context("Failed to collect memory entries")?;

        Ok(memory_entries)
    }

    pub fn delete_memory(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM memory WHERE id = ?1", params![id])
            .context("Failed to delete memory")?;
        Ok(())
    }

    pub fn clear_all_memory(&self) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM memory", [])
            .context("Failed to clear all memory")?;
        Ok(())
    }

    // Action log operations
    pub fn log_action(&self, action: &str, tool: &str, result: &str) -> Result<()> {
        let conn = self.get_connection()?;
        let timestamp = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO action_log (timestamp, action, tool, result) VALUES (?1, ?2, ?3, ?4)",
            params![timestamp, action, tool, result],
        ).context("Failed to log action")?;
        Ok(())
    }

    pub fn get_action_log(&self, limit: i32) -> Result<Vec<(i64, String, String, String)>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT timestamp, action, tool, result FROM action_log ORDER BY timestamp DESC LIMIT ?1"
        )?;

        let actions = stmt.query_map([limit], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?.collect::<Result<Vec<_>, _>>()
            .context("Failed to collect action log")?;

        Ok(actions)
    }

    pub fn clear_action_log(&self) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM action_log", [])
            .context("Failed to clear action log")?;
        Ok(())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new().expect("Failed to initialize storage")
    }
}
