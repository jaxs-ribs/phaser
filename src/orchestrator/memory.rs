use rusqlite::{Connection, Result, params};
use std::error::Error;
use std::path::Path;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
        }
    }
}

impl std::str::FromStr for MessageRole {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            "system" => Ok(MessageRole::System),
            _ => Err(format!("Invalid message role: {}", s).into()),
        }
    }
}

#[derive(Debug)]
pub struct MemoryManager {
    conn: Connection,
}

impl MemoryManager {
    /// Create a new MemoryManager and initialize the database
    pub fn new(db_path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let db_path = db_path.unwrap_or("memory.sqlite");
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(db_path)?;
        
        let mut manager = MemoryManager { conn };
        manager.initialize_database()?;
        
        Ok(manager)
    }
    
    /// Create an in-memory database (for testing)
    pub fn new_in_memory() -> Result<Self, Box<dyn Error>> {
        let conn = Connection::open(":memory:")?;
        let mut manager = MemoryManager { conn };
        manager.initialize_database()?;
        Ok(manager)
    }
    
    /// Initialize the database schema
    fn initialize_database(&mut self) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create index on timestamp for efficient querying
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversations_timestamp 
             ON conversations(timestamp)",
            [],
        )?;
        
        println!("📚 Memory database initialized");
        Ok(())
    }
    
    /// Add a new message to the conversation history
    pub fn add_message(&mut self, role: MessageRole, content: &str) -> Result<i64, Box<dyn Error>> {
        let timestamp = Utc::now();
        let timestamp_str = timestamp.to_rfc3339();
        
        let id = self.conn.execute(
            "INSERT INTO conversations (timestamp, role, content) VALUES (?1, ?2, ?3)",
            params![timestamp_str, role.to_string(), content],
        )?;
        
        println!("💬 Added {} message: {} chars", role, content.len());
        Ok(id as i64)
    }
    
    /// Get recent conversation history, limited by number of messages
    pub fn get_recent_history(&self, limit: usize) -> Result<Vec<ConversationMessage>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, role, content 
             FROM conversations 
             ORDER BY timestamp DESC 
             LIMIT ?1"
        )?;
        
        let message_iter = stmt.query_map(params![limit], |row| {
            let timestamp_str: String = row.get(1)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|_e| rusqlite::Error::InvalidColumnType(1, "timestamp".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let role_str: String = row.get(2)?;
            let role = role_str.parse::<MessageRole>()
                .map_err(|_e| rusqlite::Error::InvalidColumnType(2, "role".to_string(), rusqlite::types::Type::Text))?;
            
            Ok(ConversationMessage {
                id: row.get(0)?,
                timestamp,
                role,
                content: row.get(3)?,
            })
        })?;
        
        let mut messages = Vec::new();
        for message in message_iter {
            messages.push(message?);
        }
        
        // Reverse to get chronological order (oldest first)
        messages.reverse();
        
        Ok(messages)
    }
    
    /// Get conversation history within a time range
    pub fn get_history_by_time_range(
        &self, 
        start: DateTime<Utc>, 
        end: DateTime<Utc>
    ) -> Result<Vec<ConversationMessage>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, role, content 
             FROM conversations 
             WHERE timestamp BETWEEN ?1 AND ?2
             ORDER BY timestamp ASC"
        )?;
        
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();
        
        let message_iter = stmt.query_map(params![start_str, end_str], |row| {
            let timestamp_str: String = row.get(1)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|_e| rusqlite::Error::InvalidColumnType(1, "timestamp".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let role_str: String = row.get(2)?;
            let role = role_str.parse::<MessageRole>()
                .map_err(|_e| rusqlite::Error::InvalidColumnType(2, "role".to_string(), rusqlite::types::Type::Text))?;
            
            Ok(ConversationMessage {
                id: row.get(0)?,
                timestamp,
                role,
                content: row.get(3)?,
            })
        })?;
        
        let mut messages = Vec::new();
        for message in message_iter {
            messages.push(message?);
        }
        
        Ok(messages)
    }
    
    /// Search for messages containing specific text
    pub fn search_messages(&self, query: &str, limit: usize) -> Result<Vec<ConversationMessage>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, role, content 
             FROM conversations 
             WHERE content LIKE ?1
             ORDER BY timestamp DESC 
             LIMIT ?2"
        )?;
        
        let search_pattern = format!("%{}%", query);
        
        let message_iter = stmt.query_map(params![search_pattern, limit], |row| {
            let timestamp_str: String = row.get(1)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|_e| rusqlite::Error::InvalidColumnType(1, "timestamp".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let role_str: String = row.get(2)?;
            let role = role_str.parse::<MessageRole>()
                .map_err(|_e| rusqlite::Error::InvalidColumnType(2, "role".to_string(), rusqlite::types::Type::Text))?;
            
            Ok(ConversationMessage {
                id: row.get(0)?,
                timestamp,
                role,
                content: row.get(3)?,
            })
        })?;
        
        let mut messages = Vec::new();
        for message in message_iter {
            messages.push(message?);
        }
        
        Ok(messages)
    }
    
    /// Get conversation statistics
    pub fn get_stats(&self) -> Result<MemoryStats, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                COUNT(*) as total_messages,
                COUNT(CASE WHEN role = 'user' THEN 1 END) as user_messages,
                COUNT(CASE WHEN role = 'assistant' THEN 1 END) as assistant_messages,
                MIN(timestamp) as first_message,
                MAX(timestamp) as last_message
             FROM conversations"
        )?;
        
        let row = stmt.query_row([], |row| {
            let first_str: Option<String> = row.get(3)?;
            let last_str: Option<String> = row.get(4)?;
            
            let first_message = if let Some(ts) = first_str {
                Some(DateTime::parse_from_rfc3339(&ts)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(3, "first_message".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc))
            } else {
                None
            };
            
            let last_message = if let Some(ts) = last_str {
                Some(DateTime::parse_from_rfc3339(&ts)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(4, "last_message".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc))
            } else {
                None
            };
            
            Ok(MemoryStats {
                total_messages: row.get(0)?,
                user_messages: row.get(1)?,
                assistant_messages: row.get(2)?,
                first_message,
                last_message,
            })
        })?;
        
        Ok(row)
    }
    
    /// Clear all conversation history (use with caution!)
    pub fn clear_history(&mut self) -> Result<(), Box<dyn Error>> {
        self.conn.execute("DELETE FROM conversations", [])?;
        println!("🗑️  Cleared all conversation history");
        Ok(())
    }
    
    /// Create a formatted context string from recent messages for LLM prompts
    pub fn format_context(&self, limit: usize) -> Result<String, Box<dyn Error>> {
        let messages = self.get_recent_history(limit)?;
        
        if messages.is_empty() {
            return Ok("No previous conversation history.".to_string());
        }
        
        let mut context = String::from("Previous conversation:\n");
        for message in messages {
            context.push_str(&format!("{}: {}\n", message.role, message.content));
        }
        
        Ok(context)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_messages: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub first_message: Option<DateTime<Utc>>,
    pub last_message: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_memory_manager_creation() {
        let memory = MemoryManager::new_in_memory().unwrap();
        let stats = memory.get_stats().unwrap();
        assert_eq!(stats.total_messages, 0);
    }

    #[test]
    fn test_add_and_retrieve_messages() {
        let mut memory = MemoryManager::new_in_memory().unwrap();
        
        memory.add_message(MessageRole::User, "Hello, world!").unwrap();
        memory.add_message(MessageRole::Assistant, "Hello! How can I help you?").unwrap();
        
        let history = memory.get_recent_history(10).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].content, "Hello, world!");
        assert_eq!(history[1].content, "Hello! How can I help you?");
    }

    #[test]
    fn test_search_messages() {
        let mut memory = MemoryManager::new_in_memory().unwrap();
        
        memory.add_message(MessageRole::User, "How do I implement a function?").unwrap();
        memory.add_message(MessageRole::Assistant, "You can implement a function like this...").unwrap();
        memory.add_message(MessageRole::User, "What about variables?").unwrap();
        
        let results = memory.search_messages("function", 10).unwrap();
        assert_eq!(results.len(), 2); // Should find both the user question and assistant response
    }

    #[test]
    fn test_message_limit() {
        let mut memory = MemoryManager::new_in_memory().unwrap();
        
        for i in 0..10 {
            memory.add_message(MessageRole::User, &format!("Message {}", i)).unwrap();
        }
        
        let history = memory.get_recent_history(5).unwrap();
        assert_eq!(history.len(), 5);
        // Should get the 5 most recent messages (5-9)
        assert_eq!(history[0].content, "Message 5");
        assert_eq!(history[4].content, "Message 9");
    }

    #[test]
    fn test_context_formatting() {
        let mut memory = MemoryManager::new_in_memory().unwrap();
        
        memory.add_message(MessageRole::User, "Hello").unwrap();
        memory.add_message(MessageRole::Assistant, "Hi there!").unwrap();
        
        let context = memory.format_context(5).unwrap();
        assert!(context.contains("user: Hello"));
        assert!(context.contains("assistant: Hi there!"));
    }
}