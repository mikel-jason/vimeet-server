//! `WebSocketServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `WebSocketServer`.

use actix::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

/// web socket server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct Elevate {
    pub object: usize,
    pub owner_id: usize,
    pub room_name: String,
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct Recede {
    pub object: usize,
    pub owner_id: usize,
    pub room_name: String,
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
    polls: Vec<Poll>,
    connected: HashMap<usize, User>,
}

#[derive(Clone, Serialize)]
struct User {
    name: String,
    elevated: bool,
}

impl Default for Room {
    fn default() -> Room {
        Room {
            raised: Vec::new(),
            polls: Vec::new(),
            connected: HashMap::new(),
        }
    }
}

impl Room {
    fn remove_user(&mut self, user_id: &usize) {
        self.raised.retain(|elem| &elem.owner_id != user_id);
    }

    fn is_elevated(&self, user_id: &usize) -> Result<bool, &'static str> {
        match self.connected.get(user_id) {
            None => Err(""),
            Some(user) => Ok(user.elevated),
        }
    }

    fn set_elevated(&mut self, user_id: &usize, elevated: bool) {
        match self.connected.get_mut(user_id) {
            None => {
                return;
            }
            Some(connected) => connected.elevated = elevated,
        }
    }
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct PollCloseHelper {
    pub sender_id: usize,
    pub sender_name: String,
    pub room_name: String,
    pub poll_title: String,
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct PollVoteHelper {
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
    pub poll_title: String,
    pub option_title: String,
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct PollOption {
    pub title: String,
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
    pub poll_title: String,
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct Poll {
    pub title: String,
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
    pub options: Vec<PollOption>,
    pub votes: HashMap<usize, String>, // HashMap<user_id, option_title>
    pub closed: bool,
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

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct Instant {
    pub object: String,
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub addr: Recipient<Message>,
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
}

impl Default for WebSocketServer {
    fn default() -> WebSocketServer {
        let rooms = HashMap::new(); // mut?!

        WebSocketServer {
            sessions: HashMap::new(),
            rooms,
        }
    }
}

impl WebSocketServer {
    /// send a message to some users in a room
    ///
    /// expect of the user given in the argument `skip_id`
    ///
    /// # Arguments
    ///
    /// * `room` - a string slice with the name of the room where the message has to be send
    /// * `message` - a string slice that holds the message to be send
    /// * `skip_id` - the user id of the user that should not receive the message
    fn send_message_skip_user(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(room) = self.rooms.get(room) {
            let sessions = &room.connected;
            for (id, _) in sessions {
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

    /// send a message to all users in a room
    ///
    /// This function uses `the send_message_skip_user()-function` with the `skip_user-argument` 0.
    ///
    /// # Arguments
    ///
    /// * `room` - a string slice with the name of the room where the message has to be send
    /// * `message` - a string slice that holds the message to be send
    fn send_message_all(&self, room: &str, message: &str) {
        self.send_message_skip_user(room, message, 0);
    }

    /// send a message to a specific users in a room
    ///
    /// # Arguments
    ///
    /// * `room` - a string slice with the name of the room where the message has to be send
    /// * `message` - a string slice that holds the message to be send
    /// * `user_id` - the user id of the user that should receive the message
    fn send_message_user(&self, room: &str, message: &str, user_id: usize) {
        if let Some(room) = self.rooms.get(room) {
            let sessions = &room.connected;
            for (id, _) in sessions {
                if id == &user_id {
                    if let Some(addr) = self.sessions.get(id) {
                        let _ = addr.do_send(Message(message.to_owned()));
                    }
                    break;
                }
            }
        } else {
            println!("No room '{}' found", room);
        }
    }

    /// send a message to all elevated users in a room
    ///
    /// This function loops threw all users in the given room and sends the given message to every user that has `elevated` set to `true`.
    ///
    /// # Arguments
    ///
    /// * `room` - a string slice with the name of the room where the message has to be send
    /// * `message` - a string slice that holds the message to be send
    fn send_message_all_elevated(&self, room: &str, message: &str) {
        if let Some(room) = self.rooms.get(room) {
            let sessions = &room.connected;
            for (id, user) in sessions {
                if user.elevated {
                    if let Some(addr) = self.sessions.get(id) {
                        let _ = addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        } else {
            println!("No room '{}' found", room);
        }
    }

    /// send a message to all non-elevated users in a room
    ///
    /// This function loops threw all users in the given room and sends the given message to every user that has `elevated` set to `false`.
    ///
    /// # Arguments
    ///
    /// * `room` - a string slice with the name of the room where the message has to be send
    /// * `message` - a string slice that holds the message to be send
    fn send_message_all_not_elevated(&self, room: &str, message: &str) {
        if let Some(room) = self.rooms.get(room) {
            let sessions = &room.connected;
            for (id, user) in sessions {
                if !user.elevated {
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

/// Handler for Disconnect message.
impl Handler<Disconnect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let mut rooms_leaving: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from rooms
            for (name, rooms) in &mut self.rooms {
                if rooms.connected.remove_entry(&msg.id).is_some() {
                    rooms_leaving.push(name.to_owned());
                    break;
                }
            }
        }
        // send message to other users
        for room_name in rooms_leaving {
            let room = self
                .rooms
                .entry(room_name.clone())
                .or_insert(Room::default());
            room.remove_user(&msg.id);

            let txt = json!({
                "type": "all",
                "raised": room.raised,
                "joined": room.connected,
            })
            .to_string();
            self.send_message_all(&room_name, txt.as_str());
        }

        // TODO: remove votes (from closed polls only)
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message_skip_user(&msg.room, msg.msg.as_str(), msg.id);
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join {
            addr,
            user_id,
            user_name,
            room_name,
        } = msg;

        self.sessions.insert(user_id, addr);

        let room = self
            .rooms
            .entry(room_name.clone())
            .or_insert(Room::default());

        let elevated = if room.connected.len() > 0 {
            false
        } else {
            true
        };
        room.connected.insert(
            user_id,
            User {
                name: user_name.clone(),
                elevated,
            },
        );

        let msg = json!({
            "type": "joined",
            "object" : {
                "name": user_name,
                "id": user_id,
                "elevated": elevated
            }
        })
        .to_string();

        self.send_message_skip_user(&room_name, msg.as_str(), user_id);

        let room = self.rooms.get(&room_name).unwrap();

        let msg = json!({
            "type": "all",
            "raised": room.raised,
            "joined": room.connected,
        })
        .to_string();

        self.send_message_user(&room_name, msg.as_str(), user_id);

        let msg = json!({
            "type": "self",
            "id": user_id,
        })
        .to_string();

        self.send_message_user(&room_name, msg.as_str(), user_id);

        // TODO: add polls with options and votes
    }
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

        let elevated = self
            .rooms
            .get(msg.room_name.as_str())
            .unwrap()
            .is_elevated(&msg.owner_id)
            .unwrap_or(false);

        let txt = json!({
            "type": "raised",
            "owner_id": msg.owner_id,
            "owner_name": msg.owner_name,
            "object": &msg.object,
            "elevated": elevated,
        });
        self.send_message_all(msg.room_name.as_str(), &txt.to_string());

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

        let elevated = self
            .rooms
            .get(msg.room_name.as_str())
            .unwrap()
            .is_elevated(&msg.owner_id)
            .unwrap_or(false);

        let txt = json!({
            "type": "lower",
            "owner_id": msg.owner_id,
            "owner_name": msg.owner_name,
            "object": msg.object,
            "elevated": elevated,
        });
        self.send_message_all(&msg.room_name, &txt.to_string());
    }
}

impl Handler<Instant> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Instant, _: &mut Context<Self>) {
        let elevated = self
            .rooms
            .get(msg.room_name.as_str())
            .unwrap()
            .is_elevated(&msg.owner_id)
            .unwrap_or(false);

        let txt = json!({
            "type": "instant",
            "owner_id": msg.owner_id,
            "owner_name": msg.owner_name,
            "object": msg.object,
            "elevated": elevated,
        })
        .to_string();
        self.send_message_all(&msg.room_name, &txt);
    }
}

impl Handler<Poll> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, poll: Poll, _: &mut Context<Self>) {
        // get room
        let room = self
            .rooms
            .entry(poll.room_name.clone())
            .or_insert(Room::default());

        // check if user is elevated
        let mut user_is_elevated = room.connected.clone();
        user_is_elevated.retain(|id, user| {
            id == &poll.owner_id && &user.name == &poll.owner_name && user.elevated
        });

        if user_is_elevated.len() == 0 {
            println!("User does not have permission to create polls (not elevated)!");
            return;
        }

        // check if poll already exists
        let mut poll_exists = room.polls.clone();
        poll_exists.retain(|elem| &elem.title == &poll.title);

        if poll_exists.len() > 0 {
            println!("Poll with that title already exists!");
            return;
        }

        // clone later needed values
        let poll_title = poll.title.clone();
        let room_name = poll.room_name.clone();

        // add poll to room
        room.polls.push(poll);

        // send poll message to clients
        let txt = json!({
            "type": "poll",
            "pollobject": poll_title,
        })
        .to_string();
        self.send_message_all(&room_name, &txt);
    }
}

impl Handler<PollOption> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, poll_option: PollOption, _: &mut Context<Self>) {
        // get room
        let room = self
            .rooms
            .entry(poll_option.room_name.clone())
            .or_insert(Room::default());

        // check if user is elevated
        let mut user_is_elevated = room.connected.clone();
        user_is_elevated.retain(|id, user| {
            id == &poll_option.owner_id && &user.name == &poll_option.owner_name && user.elevated
        });

        if user_is_elevated.len() == 0 {
            println!("User does not have permission to add poll options (not elevated)!");
            return;
        }

        // check if poll exists
        let mut poll_exists = room.polls.clone();
        poll_exists.retain(|poll| poll.title == poll_option.poll_title);

        if poll_exists.len() == 0 {
            println!("Poll with that title doesn't exist!");
            return;
        }

        // get poll
        let poll_index = room
            .polls
            .iter()
            .position(|poll| poll.title == poll_option.poll_title)
            .unwrap();
        let poll = room.polls.get_mut(poll_index).unwrap();

        // check if poll is closed
        if poll.closed {
            println!("Poll is already closed!");
        }

        // check if poll_option already exists
        let mut poll_option_exists = poll.options.clone();
        poll_option_exists
            .retain(|existing_poll_option| existing_poll_option.title == poll_option.title);

        if poll_option_exists.len() > 0 {
            println!("Poll-Option with that title in this poll does already exist!");
            return;
        }

        // clone later needed values
        let poll_option_title = poll_option.title.clone();
        let room_name = poll_option.room_name.clone();

        // add poll_option to poll
        poll.options.push(poll_option);

        // send poll option message to clients
        let txt = json!({
            "type": "polloption",
            "pollobject": poll.title,
            "polloptionobject": poll_option_title,
        })
        .to_string();
        self.send_message_all(&room_name, &txt);
    }
}

impl Handler<PollVoteHelper> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, vote: PollVoteHelper, _: &mut Context<Self>) {
        // get room
        let room = self
            .rooms
            .entry(vote.room_name.clone())
            .or_insert(Room::default());

        // check if poll exists
        let mut poll_exists = room.polls.clone();
        poll_exists.retain(|elem| &elem.title == &vote.poll_title);

        if poll_exists.len() == 0 {
            println!("Poll with that title doesn't exist!");
            return;
        }

        // get poll
        let poll_index = room
            .polls
            .iter()
            .position(|poll| poll.title == vote.poll_title)
            .unwrap();
        let poll = room.polls.get_mut(poll_index).unwrap();

        // check if poll is closed
        if poll.closed {
            println!("Poll is already closed!");
        }

        // check if poll_option exists
        let mut poll_option_exists = poll.options.clone();
        poll_option_exists
            .retain(|existing_poll_option| existing_poll_option.title == vote.option_title);

        if poll_option_exists.len() == 0 {
            println!("Poll-Option with that title in this poll doesn't exist!");
            return;
        }

        // check if user has already voted
        if poll.votes.contains_key(&vote.owner_id) {
            // remove existing vote
            println!(
                "User has already votes in this poll, removing existing vote and adding new vote."
            );
            poll.votes.remove(&vote.owner_id);

            // TODO: inform user about vote remove
        }

        // clone later needed values
        let poll_option_title = vote.poll_title.clone();

        // add vote to poll
        poll.votes.insert(vote.owner_id, vote.option_title);

        // send poll option message to clients
        let elevated_txt = json!({
            "type": "vote",
            "pollobject": poll.title,
            "polloptionobject": poll_option_title,
            "username": vote.owner_name,
        })
        .to_string();

        let not_elevated_txt = json!({
            "type": "vote",
            "pollobject": poll.title,
            "polloptionobject": poll_option_title,
        })
        .to_string();

        self.send_message_all_elevated(&vote.room_name, &elevated_txt);
        self.send_message_all_not_elevated(&vote.room_name, &not_elevated_txt);
    }
}

impl Handler<PollCloseHelper> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, close: PollCloseHelper, _: &mut Context<Self>) {
        // get room
        let room = self
            .rooms
            .entry(close.room_name.clone())
            .or_insert(Room::default());

        // check if poll exists
        let mut poll_exists = room.polls.clone();
        poll_exists.retain(|elem| &elem.title == &close.poll_title);

        if poll_exists.len() == 0 {
            println!("Poll with that title doesn't exist!");
            return;
        }

        // get poll
        let poll_index = room
            .polls
            .iter()
            .position(|poll| poll.title == close.poll_title)
            .unwrap();
        let poll = room.polls.get_mut(poll_index).unwrap();

        // check if poll is closed
        if poll.closed {
            println!("Poll is already closed!");
        }

        // close poll
        poll.closed = true;

        // send poll option message to clients
        let txt = json!({
            "type": "closepoll",
            "pollobject": poll.title,
        })
        .to_string();
        self.send_message_all(&close.room_name, &txt);
    }
}

impl WebSocketServer {
    /// Handles managing priligiges on request
    ///
    /// # Arguments
    /// * `room_name` - The room in which the user's priviliges should be changed
    /// * `requested_id` - The user who requests the change. Elevated priviliges needed.
    /// * `user_id` - The user whose priviliges should be changed.
    /// * `elevated` - If the user should have elevated priviliges or not.
    fn process_priviliges(
        &mut self,
        room_name: &String,
        requester_id: usize,
        user_id: usize,
        elevated: bool,
    ) -> Result<(), &'static str> {
        if let Some(room) = self.rooms.get_mut(room_name) {
            if room.is_elevated(&requester_id)? && room.is_elevated(&user_id)? != elevated {
                room.set_elevated(&user_id, elevated);
                return Ok(());
            }
        }
        Err("")
    }
}

impl Handler<Elevate> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Elevate, _: &mut Context<Self>) {
        match self.process_priviliges(&msg.room_name, msg.owner_id, msg.object, true) {
            Err(_) => (),
            Ok(_) => {
                let txt = json!({
                    "type": "elevated",
                    "object": msg.object,
                })
                .to_string();
                self.send_message_all(&msg.room_name, &txt);
            }
        }
    }
}

impl Handler<Recede> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Recede, _: &mut Context<Self>) {
        match self.process_priviliges(&msg.room_name, msg.owner_id, msg.object, false) {
            Err(_) => (),
            Ok(_) => {
                let txt = json!({
                    "type": "receded",
                    "object": msg.object,
                })
                .to_string();
                self.send_message_all(&msg.room_name, &txt);
            }
        }
    }
}
