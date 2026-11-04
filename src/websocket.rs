//! WebSocket Support Module
//!
//! Provides WebSocket functionality for real-time communication.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// WebSocket connection ID
pub type ConnectionId = String;

/// WebSocket message types
#[derive(Debug, Clone)]
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close,
}

/// WebSocket connection handler trait
pub trait WsHandler: Send + Sync {
    /// Called when a new connection is established
    fn on_open(&self, conn_id: &ConnectionId);

    /// Called when a message is received
    fn on_message(&self, conn_id: &ConnectionId, message: WsMessage);

    /// Called when a connection is closed
    fn on_close(&self, conn_id: &ConnectionId);

    /// Called when an error occurs
    fn on_error(&self, conn_id: &ConnectionId, error: String);
}

/// WebSocket room for group messaging
#[derive(Debug, Clone)]
pub struct WsRoom {
    name: String,
    members: Arc<RwLock<Vec<ConnectionId>>>,
}

impl WsRoom {
    /// Create a new room
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            members: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Join the room
    pub fn join(&self, conn_id: ConnectionId) {
        let mut members = self.members.write();
        if !members.contains(&conn_id) {
            members.push(conn_id);
        }
    }

    /// Leave the room
    pub fn leave(&self, conn_id: &ConnectionId) {
        let mut members = self.members.write();
        members.retain(|id| id != conn_id);
    }

    /// Get all members
    pub fn members(&self) -> Vec<ConnectionId> {
        self.members.read().clone()
    }

    /// Get room name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get member count
    pub fn count(&self) -> usize {
        self.members.read().len()
    }
}

/// WebSocket server for managing connections
#[derive(Clone)]
pub struct WsServer {
    connections: Arc<RwLock<HashMap<ConnectionId, mpsc::Sender<WsMessage>>>>,
    rooms: Arc<RwLock<HashMap<String, WsRoom>>>,
}

impl WsServer {
    /// Create a new WebSocket server
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new connection
    pub fn register(&self, conn_id: ConnectionId, sender: mpsc::Sender<WsMessage>) {
        let mut connections = self.connections.write();
        connections.insert(conn_id, sender);
    }

    /// Unregister a connection
    pub fn unregister(&self, conn_id: &ConnectionId) {
        let mut connections = self.connections.write();
        connections.remove(conn_id);

        // Remove from all rooms
        let rooms = self.rooms.read();
        for room in rooms.values() {
            room.leave(conn_id);
        }
    }

    /// Send message to a specific connection
    pub async fn send_to(&self, conn_id: &ConnectionId, message: WsMessage) -> bool {
        let connections = self.connections.read();
        if let Some(sender) = connections.get(conn_id) {
            sender.send(message).await.is_ok()
        } else {
            false
        }
    }

    /// Broadcast message to all connections
    pub async fn broadcast(&self, message: WsMessage) {
        let connections = self.connections.read();
        for sender in connections.values() {
            let _ = sender.send(message.clone()).await;
        }
    }

    /// Broadcast to a specific room
    pub async fn broadcast_to_room(&self, room_name: &str, message: WsMessage) {
        let rooms = self.rooms.read();
        if let Some(room) = rooms.get(room_name) {
            let members = room.members();
            let connections = self.connections.read();

            for member_id in members {
                if let Some(sender) = connections.get(&member_id) {
                    let _ = sender.send(message.clone()).await;
                }
            }
        }
    }

    /// Create or get a room
    pub fn room(&self, name: &str) -> WsRoom {
        let mut rooms = self.rooms.write();
        rooms
            .entry(name.to_string())
            .or_insert_with(|| WsRoom::new(name))
            .clone()
    }

    /// Join a room
    pub fn join_room(&self, room_name: &str, conn_id: ConnectionId) {
        let room = self.room(room_name);
        room.join(conn_id);
    }

    /// Leave a room
    pub fn leave_room(&self, room_name: &str, conn_id: &ConnectionId) {
        let rooms = self.rooms.read();
        if let Some(room) = rooms.get(room_name) {
            room.leave(conn_id);
        }
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.read().len()
    }

    /// Get all connection IDs
    pub fn connections(&self) -> Vec<ConnectionId> {
        self.connections.read().keys().cloned().collect()
    }
}

impl Default for WsServer {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket configuration
#[derive(Debug, Clone)]
pub struct WsConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Ping interval in seconds
    pub ping_interval: u64,
    /// Connection timeout in seconds
    pub timeout: u64,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            max_message_size: 64 * 1024, // 64KB
            ping_interval: 30,
            timeout: 60,
        }
    }
}
