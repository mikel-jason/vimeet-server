//! `WebSocketServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `WebSocketServer`.

use actix::prelude::*;
use serde::Serialize;
use serde_json::{json, Value as Arbitrary};
use std::collections::HashMap;

use crate::messages;

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

/// The room object
/// # Parameters
/// * `raised` - A list of all raised objects in this room
/// * `polls` - A list of all poll objects in this room
/// * `connected` - A HashMap with all users in this room: <userid: usize, user: User>
#[derive(Clone)]
pub struct Room {
    raised: Vec<Raised>,
    polls: Vec<Poll>,
    connected: HashMap<usize, User>,
}

/// The user object
/// # Parameters
/// * `name` - The name of the user
/// * `elevated` - Bool: if the user is elevated
#[derive(Clone, Serialize)]
pub struct User {
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
    /// remove a user from a room
    ///
    /// # Arguments
    /// * `user_id` - the id of the user that should be removed
    fn remove_user(&mut self, user_id: &usize) {
        self.raised.retain(|elem| &elem.owner_id != user_id);
    }

    /// returns a Result with the information if an user is elevated or not
    ///
    /// # Arguments
    /// * `user_id` - the id of the user you want the elevated information from
    fn is_elevated(&self, user_id: &usize) -> Result<bool, &'static str> {
        match self.connected.get(user_id) {
            None => Err(""),
            Some(user) => Ok(user.elevated),
        }
    }

    /// set the elevated state of an user
    ///
    /// # Arguments
    /// * `user_id` - the id of the user you want to set the elevated state
    /// * `elevated` - the elevated state (true / false)
    fn set_elevated(&mut self, user_id: &usize, elevated: bool) {
        match self.connected.get_mut(user_id) {
            None => {
                return;
            }
            Some(connected) => connected.elevated = elevated,
        }
    }
}

/// A helper object to close a poll
///
/// # Parameters
/// * `sender_id` - the id of the user who sends the message
/// * `sender_name` - the name of the user who sends the message
/// * `room_name` - the name of the room in which the user sends the message
/// * `poll_title` - the name of the poll the user wants to close
#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct PollCloseHelper {
    pub sender_id: usize,
    pub sender_name: String,
    pub room_name: String,
    pub poll_title: String,
}

/// A helper object to vote on a poll (-option)
///
/// # Parameters
/// * `owner_id` - the id of the user who sends the message
/// * `owner_name` - the name of the user who sends the message
/// * `room_name` - the name of the room in which the user sends the message
/// * `poll_title` - the name of the poll the user wants to vote on
/// * `option_title` - the name of the option the user wants to vote on
#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct PollVoteHelper {
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
    pub poll_title: String,
    pub option_title: String,
}

/// The poll option object
///
/// # Parameters
/// * `title` - the title of the poll option
/// * `owner_id` - the id of the user that created this option
/// * `owner_name` - the name of the user that created this option
/// * `room_name` - the name of the room in which this option (and the poll) was created
/// * `poll_title` - the name of the poll this option belongs to
#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct PollOption {
    pub title: String,
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
    pub poll_title: String,
}

/// The poll option object
///
/// # Parameters
/// * `title` - the title of the poll
/// * `owner_id` - the id of the user that created this poll
/// * `owner_name` - the name of the user that created this poll
/// * `room_name` - the name of the room in which this poll was created
/// * `options` - a list of PollOptions
/// * `votes` - a HashMap of votes: <userid: usize, option_title: String>
#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct Poll {
    pub title: String,
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
    pub options: Vec<PollOption>,
    pub votes: HashMap<usize, String>,
    pub closed: bool,
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct Raise {
    pub object: Arbitrary,
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct Raised {
    pub object: Arbitrary,
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
    pub object: Arbitrary,
    pub owner_id: usize,
    pub owner_name: String,
    pub room_name: String,
}

#[derive(Message, Serialize, Clone, Debug)]
#[rtype(result = "()")]
pub struct Instant {
    pub object: Arbitrary,
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
    /// * `room` - a string slice with the name of the room where the message has to be send
    /// * `message` - a string slice that holds the message to be send
    fn send_message_all(&mut self, room: &str, message: &str) {
        self.send_message_skip_user(room, message, 0);
    }

    /// send a message to a specific users in a room
    ///
    /// # Arguments
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

    /// send an error message to a specific users in a room
    ///
    /// This function loops threw all users in the given room and sends the given message to every user that has `elevated` set to `false`.
    ///
    /// # Arguments
    /// * `room` - a string slice with the name of the room where the message has to be send
    /// * `error_code` - a string slice with a short error name
    /// * `error_description` - a string slice with a longer description what went wrong
    /// * `user_id` - the user id of the user that should receive the message
    fn send_error_user(
        &self,
        room: &str,
        error_code: &str,
        error_description: &str,
        user_id: usize,
    ) {
        let error_message = json!(messages::outbound::Error {
            r#type: messages::outbound::Types::Error,
            object: error_code.to_string(),
            description: error_description.to_string(),
        })
        .to_string();
        self.send_message_user(room, &error_message, user_id);
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
            for (room_name, room) in &mut self.rooms {
                if room.connected.remove_entry(&msg.id).is_some() {
                    rooms_leaving.push(room_name.to_owned());
                    room.remove_user(&msg.id);
                    break;
                }
            }

            for room_name in rooms_leaving {
                let room = self
                    .rooms
                    .entry(room_name.clone())
                    .or_insert(Room::default());

                let txt = json!(messages::outbound::All {
                    r#type: messages::outbound::Types::All,
                    raised: room.raised.clone(),
                    joined: room.connected.clone(),
                })
                .to_string();

                self.send_message_all(&room_name, txt.as_str());

                let room = self
                    .rooms
                    .entry(room_name.clone())
                    .or_insert(Room::default());

                // get username
                let user_id = msg.id;

                let mut messages_to_send_to_elevated: Vec<String> = Vec::new();
                let mut messages_to_send_to_not_elevated: Vec<String> = Vec::new();

                for i in 0..room.polls.clone().len() {
                    let poll = room.polls[i].clone();
                    if !poll.closed {
                        for (id, poll_option_title) in poll.votes {
                            if id == msg.id {
                                // delete vote
                                room.polls[i].votes.remove(&id);

                                // send poll option message to clients
                                let elevated_txt = json!(messages::outbound::VoteDelete {
                                    r#type: messages::outbound::Types::VoteDelete,
                                    pollobject: poll.title.clone(),
                                    polloptionobject: poll_option_title.clone(),
                                    userid: user_id,
                                })
                                .to_string();
                                let not_elevated_txt = json!(messages::outbound::VoteDelete {
                                    r#type: messages::outbound::Types::VoteDelete,
                                    pollobject: poll.title.clone(),
                                    polloptionobject: poll_option_title.clone(),
                                    userid: 0,
                                })
                                .to_string();

                                messages_to_send_to_elevated.push(elevated_txt);
                                messages_to_send_to_not_elevated.push(not_elevated_txt);
                            }
                        }
                    }
                }

                for message_to_send_to_elevated in messages_to_send_to_elevated {
                    self.send_message_all_elevated(&room_name, &message_to_send_to_elevated);
                }

                for message_to_send_to_not_elevated in messages_to_send_to_not_elevated {
                    self.send_message_all_not_elevated(
                        &room_name,
                        &message_to_send_to_not_elevated,
                    );
                }
            }
        }
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

        let msg = json!(messages::outbound::User {
            r#type: messages::outbound::Types::Joined,
            object: messages::outbound::UserFormat {
                id: user_id,
                name: user_name,
                elevated
            }
        })
        .to_string();
        self.send_message_skip_user(&room_name, msg.as_str(), user_id);

        let room = self
            .rooms
            .entry(room_name.clone())
            .or_insert(Room::default());

        let msg = json!(messages::outbound::All {
            r#type: messages::outbound::Types::All,
            raised: room.raised.clone(),
            joined: room.connected.clone()
        })
        .to_string();

        self.send_message_user(&room_name, msg.as_str(), user_id);

        let msg = json!(messages::outbound::PermissionChange {
            r#type: messages::outbound::Types::SelfStatus,
            object: user_id,
            elevated
        })
        .to_string();

        self.send_message_user(&room_name, msg.as_str(), user_id);

        let room = self
            .rooms
            .entry(room_name.clone())
            .or_insert(Room::default());

        // send polls
        for poll in room.polls.clone() {
            if !poll.closed {
                let poll_txt = json!(messages::outbound::Poll {
                    r#type: messages::outbound::Types::Poll,
                    object: poll.title.clone(),
                })
                .to_string();
                self.send_message_user(&room_name, &poll_txt, user_id);

                // send options for poll
                for option in poll.options.clone() {
                    let option_txt = json!(messages::outbound::PollOption {
                        r#type: messages::outbound::Types::PollOption,
                        pollobject: poll.title.clone(),
                        polloptionobject: option.title.clone(),
                    })
                    .to_string();
                    self.send_message_user(&room_name, &option_txt, user_id);
                }

                // send votes for poll
                for (_, option_title) in poll.votes.clone() {
                    let vote_txt = json!(messages::outbound::Vote {
                        r#type: messages::outbound::Types::Vote,
                        pollobject: poll.title.clone(),
                        polloptionobject: option_title.clone(),
                        username: "".to_string(),
                        userid: 0,
                    })
                    .to_string();
                    self.send_message_user(&room_name, &vote_txt, user_id);
                }
            }
        }
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
            self.send_error_user(
                &msg.room_name,
                "already_raised",
                "Refusing to raise, already raised",
                msg.owner_id,
            );
            println!("Refusing to raise, already raised");
            return;
        }

        let elevated = self
            .rooms
            .get(msg.room_name.as_str())
            .unwrap()
            .is_elevated(&msg.owner_id)
            .unwrap_or(false);

        let txt = json!(messages::outbound::OwnedObject {
            r#type: messages::outbound::Types::Raised,
            owner_id: msg.owner_id,
            owner_name: msg.owner_name.clone(),
            object: msg.object.clone(),
            elevated: elevated,
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
            self.send_error_user(
                &msg.room_name,
                "not_raised",
                "Refusing to lower, is not raised",
                msg.owner_id,
            );
            println!("Refusing to lower, is not raised");
            return;
        }

        let raised_equivalent = Raised {
            object: json!(equiv_clone.object),
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

        let txt = json!(messages::outbound::OwnedObject {
            r#type: messages::outbound::Types::Lower,
            owner_id: msg.owner_id,
            owner_name: msg.owner_name,
            object: msg.object,
            elevated: elevated,
        })
        .to_string();
        self.send_message_all(&msg.room_name, &txt);
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

        let txt = json!(messages::outbound::OwnedObject {
            r#type: messages::outbound::Types::Instant,
            owner_id: msg.owner_id,
            owner_name: msg.owner_name,
            object: msg.object,
            elevated: elevated,
        })
        .to_string();

        self.send_message_all(&msg.room_name, &txt);
    }
}

/// Handler for creating polls
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
            self.send_error_user(
                &poll.room_name,
                "no_permission",
                "You do not have permission to create polls (because you're not elevated)",
                poll.owner_id,
            );
            println!("User does not have permission to create polls (not elevated)");
            return;
        }

        // check if poll already exists
        let mut poll_exists = room.polls.clone();
        poll_exists.retain(|elem| &elem.title == &poll.title);

        if poll_exists.len() > 0 {
            self.send_error_user(
                &poll.room_name,
                "poll_already_exists",
                "A poll with that title already exists",
                poll.owner_id,
            );
            println!("A poll with that title already exists");
            return;
        }

        // clone later needed values
        let poll_title = poll.title.clone();
        let room_name = poll.room_name.clone();

        // add poll to room
        room.polls.push(poll);

        // send poll message to clients
        let poll_txt = json!(messages::outbound::Poll {
            r#type: messages::outbound::Types::Poll,
            object: poll_title.clone(),
        })
        .to_string();
        self.send_message_all(&room_name, &poll_txt);
    }
}

/// Handler for creating poll options
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
            self.send_error_user(
                &poll_option.room_name,
                "no_permission",
                "You do not have permission to add poll options (because you're not elevated)",
                poll_option.owner_id,
            );
            println!("User does not have permission to add poll options (not elevated)");
            return;
        }

        // check if poll exists
        let mut poll_exists = room.polls.clone();
        poll_exists.retain(|poll| poll.title == poll_option.poll_title);

        if poll_exists.len() == 0 {
            self.send_error_user(
                &poll_option.room_name,
                "poll_does_not_exist",
                "A poll with that title doesn't exist",
                poll_option.owner_id,
            );
            println!("A poll with that title doesn't exist");
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
            self.send_error_user(
                &poll_option.room_name,
                "poll_closed",
                "Sorry, the poll is already closed",
                poll_option.owner_id,
            );
            println!("Poll is already closed");
            return;
        }

        // check if poll_option already exists
        let mut poll_option_exists = poll.options.clone();
        poll_option_exists
            .retain(|existing_poll_option| existing_poll_option.title == poll_option.title);

        if poll_option_exists.len() > 0 {
            self.send_error_user(
                &poll_option.room_name,
                "poll_option_already_exists",
                "A poll-option with that title in this poll does already exist",
                poll_option.owner_id,
            );
            println!("A poll-option with that title in this poll does already exist");
            return;
        }

        // clone later needed values
        let poll_option_title = poll_option.title.clone();
        let room_name = poll_option.room_name.clone();

        // add poll_option to poll
        poll.options.push(poll_option);

        // send poll option message to clients
        let txt = json!(messages::outbound::PollOption {
            r#type: messages::outbound::Types::PollOption,
            pollobject: poll.title.clone(),
            polloptionobject: poll_option_title.clone(),
        })
        .to_string();
        self.send_message_all(&room_name, &txt);
    }
}

/// Handler for voting
impl Handler<PollVoteHelper> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, vote: PollVoteHelper, _: &mut Context<Self>) {
        let room = self
            .rooms
            .entry(vote.room_name.clone())
            .or_insert(Room::default());

        // check if poll exists
        let mut poll_exists = room.polls.clone();
        poll_exists.retain(|elem| &elem.title == &vote.poll_title);

        if poll_exists.len() == 0 {
            self.send_error_user(
                &vote.room_name,
                "poll_does_not_exist",
                "A poll with that title doesn't exist",
                vote.owner_id,
            );
            println!("A poll with that title doesn't exist");
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
            self.send_error_user(
                &vote.room_name,
                "poll_closed",
                "Sorry, the poll is already closed",
                vote.owner_id,
            );
            println!("Poll is already closed!");
            return;
        }

        // check if poll_option exists
        let mut poll_option_exists = poll.options.clone();
        poll_option_exists
            .retain(|existing_poll_option| existing_poll_option.title == vote.option_title);

        if poll_option_exists.len() == 0 {
            self.send_error_user(
                &vote.room_name,
                "poll_option_does_not_exist",
                "A poll-option with that title in this poll doesn't exist",
                vote.owner_id,
            );
            println!("Poll-Option with that title in this poll doesn't exist");
            return;
        }

        // check if user has already voted
        let mut remove_vote = false;
        let mut remove_vote_option_title = "".to_string();

        if poll.votes.contains_key(&vote.owner_id) {
            println!(
                "User has already votes in this poll, removing existing vote and adding new vote."
            );

            // send delete vote message to clients
            for (userid, poll_option_title) in poll.votes.clone() {
                if userid == vote.owner_id {
                    remove_vote = true;
                    remove_vote_option_title = poll_option_title.to_string().clone();
                    break;
                }
            }

            // remove existing vote
            poll.votes.remove(&vote.owner_id);
        }

        // clone later needed values
        let poll_option_title = vote.option_title.clone();
        let poll_title = poll.title.clone();

        // add vote to poll
        poll.votes.insert(vote.owner_id, vote.option_title);

        // inform other users if one vote has to be removed
        if remove_vote {
            let elevated_txt = json!(messages::outbound::VoteDelete {
                r#type: messages::outbound::Types::VoteDelete,
                pollobject: poll_title.clone(),
                polloptionobject: remove_vote_option_title.clone(),
                userid: vote.owner_id,
            })
            .to_string();
            let not_elevated_txt = json!(messages::outbound::VoteDelete {
                r#type: messages::outbound::Types::VoteDelete,
                pollobject: poll_title.clone(),
                polloptionobject: remove_vote_option_title.clone(),
                userid: 0,
            })
            .to_string();

            self.send_message_all_elevated(&vote.room_name, &elevated_txt);
            self.send_message_all_not_elevated(&vote.room_name, &not_elevated_txt);
        }

        // send poll option message to clients
        let elevated_txt = json!(messages::outbound::Vote {
            r#type: messages::outbound::Types::Vote,
            pollobject: poll_title.clone(),
            polloptionobject: poll_option_title.clone(),
            username: vote.owner_name.clone(),
            userid: vote.owner_id,
        })
        .to_string();
        let not_elevated_txt = json!(messages::outbound::Vote {
            r#type: messages::outbound::Types::Vote,
            pollobject: poll_title.clone(),
            polloptionobject: poll_option_title.clone(),
            username: "".to_string(),
            userid: 0,
        })
        .to_string();

        self.send_message_all_elevated(&vote.room_name, &elevated_txt);
        self.send_message_all_not_elevated(&vote.room_name, &not_elevated_txt);
    }
}

/// Handler for closing polls
impl Handler<PollCloseHelper> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, close: PollCloseHelper, _: &mut Context<Self>) {
        // get room
        let room = self
            .rooms
            .entry(close.room_name.clone())
            .or_insert(Room::default());

        // check if user is elevated
        let mut user_is_elevated = room.connected.clone();
        user_is_elevated.retain(|id, user| {
            id == &close.sender_id && &user.name == &close.sender_name && user.elevated
        });

        if user_is_elevated.len() == 0 {
            self.send_error_user(
                &close.room_name,
                "no_permission",
                "You do not have permission to close polls (because you're not elevated)",
                close.sender_id,
            );
            println!("User does not have permission to close polls (not elevated)");
            return;
        }

        // check if poll exists
        let mut poll_exists = room.polls.clone();
        poll_exists.retain(|elem| &elem.title == &close.poll_title);

        if poll_exists.len() == 0 {
            self.send_error_user(
                &close.room_name,
                "poll_does_not_exist",
                "A poll with that title doesn't exist",
                close.sender_id,
            );
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
            self.send_error_user(
                &close.room_name,
                "poll_closed",
                "Sorry, the poll is already closed",
                close.sender_id,
            );
            println!("Poll is already closed!");
            return;
        }

        // close poll
        poll.closed = true;

        // send poll option message to clients
        let txt = json!(messages::outbound::PollClose {
            r#type: messages::outbound::Types::PollClose,
            object: poll.title.clone(),
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

                // resend votes (with user_id and user_name) for open polls
                let room_imut = room.clone();
                for poll in room_imut.polls.clone() {
                    if !poll.closed {
                        // send votes for poll
                        for (userid, option_title) in poll.votes.clone() {
                            let user = room_imut.connected.get(&userid).unwrap();

                            if elevated {
                                let del_vote_txt = json!(messages::outbound::VoteDelete {
                                    r#type: messages::outbound::Types::VoteDelete,
                                    pollobject: poll.title.clone(),
                                    polloptionobject: option_title.clone(),
                                    userid: 0,
                                })
                                .to_string();
                                self.send_message_user(&room_name, &del_vote_txt, user_id);

                                let vote_txt = json!(messages::outbound::Vote {
                                    r#type: messages::outbound::Types::Vote,
                                    pollobject: poll.title.clone(),
                                    polloptionobject: option_title.clone(),
                                    username: user.name.clone(),
                                    userid: userid,
                                })
                                .to_string();
                                self.send_message_user(&room_name, &vote_txt, user_id);
                            } else {
                                let del_vote_txt = json!(messages::outbound::VoteDelete {
                                    r#type: messages::outbound::Types::VoteDelete,
                                    pollobject: poll.title.clone(),
                                    polloptionobject: option_title.clone(),
                                    userid: userid,
                                })
                                .to_string();
                                self.send_message_user(&room_name, &del_vote_txt, user_id);

                                let vote_txt = json!(messages::outbound::Vote {
                                    r#type: messages::outbound::Types::Vote,
                                    pollobject: poll.title.clone(),
                                    polloptionobject: option_title.clone(),
                                    username: "".to_string(),
                                    userid: 0,
                                })
                                .to_string();
                                self.send_message_user(&room_name, &vote_txt, user_id);
                            }
                        }
                    }
                }

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
                let txt = json!(messages::outbound::PermissionChange {
                    r#type: messages::outbound::Types::Elevated,
                    object: msg.object,
                    elevated: true
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
                let txt = json!(messages::outbound::PermissionChange {
                    r#type: messages::outbound::Types::Receded,
                    object: msg.object,
                    elevated: false
                })
                .to_string();
                self.send_message_all(&msg.room_name, &txt);
            }
        }
    }
}
