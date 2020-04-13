//! `WebSocketServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `WebSocketServer`.

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde::{Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};

/// web socket server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for web socket server communications

/// New web socket session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}

#[derive(Clone)]
pub struct Room {
    raised: Vec<Raised>,
    connected: HashSet<usize>,
}

impl Default for Room {
    fn default() -> Room {
        Room {
            raised: Vec::new(),
            connected: HashSet::new(),
        }
    }
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct Raise {
    pub object: String,
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct Raised {
    pub object: String,
    owner_id: usize,
    owner_name: String,
}

impl std::cmp::PartialEq for Raised {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object && self.owner_id == other.owner_id
    }
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct Lower {
    pub object: String,
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
}

/// List of available rooms
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client id
    pub user_id: usize,
    pub user_name: String,
    /// Room name
    pub room_name: String,
}

/// `WebSocketServer` manages web socket rooms and responsible for coordinating web socket
/// session. implementation is super primitive
pub struct WebSocketServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, Room>,
    rng: ThreadRng,
}

impl Default for WebSocketServer {
    fn default() -> WebSocketServer {
        let rooms = HashMap::new(); // mut?!

        WebSocketServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
        }
    }
}

impl WebSocketServer {
    /// Send message to all users in the room
    fn send_message(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(room) = self.rooms.get(room) {
            let sessions = &room.connected;
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        let _ = addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        } else {
            println!("No room '{}' found", room);
        }
    }
}

/// Make actor from `WebSocketServer`
impl Actor for WebSocketServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for WebSocketServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // notify all users in same room
        self.send_message(&"Main".to_owned(), "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms_leaving: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, rooms) in &mut self.rooms {
                if rooms.connected.remove(&msg.id) {
                    rooms_leaving.push(name.to_owned());
                }
            }
        }
        // send message to other users
        for room in rooms_leaving {
            self.send_message(&room, "Someone disconnected", 0);
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.room, msg.msg.as_str(), msg.id);
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListRooms> for WebSocketServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join {
            user_id,
            user_name,
            room_name,
        } = msg;
        let mut rooms_leaving = Vec::new();
        println!("{} ({}) joining {}", user_name, user_id, room_name);

        // remove session from all rooms
        for (n, rooms) in &mut self.rooms {
            if rooms.connected.remove(&user_id) {
                rooms_leaving.push(n.to_owned());
            }
        }
        // send message to other users
        for room in rooms_leaving {
            self.send_message(&room, "Someone disconnected", 0);
        }

        self.rooms
            .entry(room_name.clone())
            .or_insert(Room::default())
            .connected
            .insert(user_id);

        let msg = json!({
            "type": "joined",
            "name": user_name,
            "id": user_id,
        })
        .to_string();

        self.send_message(&room_name, &msg, user_id);
    }
}

#[derive(Serialize)]
struct IncrementPayload {
    object: String,
    owner: usize,
}

impl Handler<Raise> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Raise, _: &mut Context<Self>) {
        let mut check_raised = self
            .rooms
            .get(msg.room_name.as_str())
            .unwrap()
            .raised
            .clone();
        check_raised.retain(|elem| elem.object == msg.object && elem.owner_id == msg.owner_id);

        if check_raised.len() > 0 {
            println!("Refusing to raise, already raised");
            return;
        }

        let txt = json!({
            "type": "raised",
            "owner_id": msg.owner_id,
            "owner_name": msg.owner_name,
            "object": &msg.object,
        });
        self.send_message(msg.room_name.as_str(), &txt.to_string(), 100000);

        let room = self.rooms.entry(msg.room_name).or_insert(Room::default());
        room.raised.push(Raised {
            object: msg.object,
            owner_id: msg.owner_id,
            owner_name: msg.owner_name,
        });
    }
}

impl Handler<Lower> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Lower, _: &mut Context<Self>) {
        let equiv_clone = msg.clone();
        let room = self
            .rooms
            .entry(msg.room_name.clone())
            .or_insert(Room::default());

        let mut check_raised = room.raised.clone();
        check_raised.retain(|elem| &elem.object == &msg.object && &elem.owner_id == &msg.owner_id);

        if check_raised.len() == 0 {
            println!("Refusing to lower, is not raised");
            return;
        }

        let raised_equivalent = Raised {
            object: equiv_clone.object,
            owner_id: equiv_clone.owner_id,
            owner_name: equiv_clone.owner_name,
        };

        room.raised.retain(|elem| elem != &raised_equivalent);

        let txt = json!({
            "type": "lower",
            "owner_id": msg.owner_id,
            "owner_name": msg.owner_name,
            "object": msg.object,
        });
        self.send_message(&msg.room_name, &txt.to_string(), 100000);
    }
}
