pub mod inbound {
    use serde::{Deserialize, Serialize};
    use serde_json::Value as Arbitrary;
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::{error, fmt};

    /// Error if message has unknown message type
    ///
    /// For all known types, see
    /// [Types](#struct.Types)
    #[derive(Debug)]
    pub struct InvalidMessageType;

    impl fmt::Display for InvalidMessageType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Invalid message type")
        }
    }

    // see https://doc.rust-lang.org/stable/rust-by-example/error/multiple_error_types/define_error_type.html
    impl error::Error for InvalidMessageType {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            None
        }
    }

    /// Get type of any message struct
    ///
    /// Ensuring that every struct representing a message skeleton implements the same basic
    /// functions to return the message type
    pub trait GetMessageType {
        fn get_type(&self) -> Result<Types, InvalidMessageType>;
    }

    /// All known types of incoming messages
    #[derive(Debug)]
    pub enum Types {
        Raise,
        Lower,
        Instant,
        Elevate,
        Recede,
        Poll,
        PollOption,
        Vote,
        PollClose,
    }

    impl FromStr for Types {
        type Err = InvalidMessageType;

        /// Get type based on string literal
        ///
        /// * `s` String representation of a type
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "raise" => Ok(Types::Raise),
                "lower" => Ok(Types::Lower),
                "instant" => Ok(Types::Instant),
                "elevate" => Ok(Types::Elevate),
                "recede" => Ok(Types::Recede),
                "poll" => Ok(Types::Poll),
                "polloption" => Ok(Types::PollOption),
                "vote" => Ok(Types::Vote),
                "closepoll" => Ok(Types::PollClose),
                _ => Err(InvalidMessageType {}),
            }
        }
    }

    /// Inbound message skeleton: Arbitrary object
    ///
    /// * `type` - Message type, see [Types](#struct.Types)
    /// * `object` - Any value a JSON parameter can hold
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ArbitraryObject {
        pub r#type: String,
        pub object: Arbitrary,
    }

    impl GetMessageType for ArbitraryObject {
        /// Get message type or error
        ///
        /// # Example
        /// ```
        /// let msg: Result<StringObject, _> = serde_json::from_str(m);
        ///     match msg {
        ///         Ok(msg) => match msg.get_type() {
        ///             Ok(Types::Raised) => ()
        ///             _ => )_
        ///         }
        ///     }
        /// ```
        fn get_type(&self) -> Result<Types, InvalidMessageType> {
            Types::from_str(self.r#type.as_str())
        }
    }

    /// Inbound message skeleton: Unsigned integer object
    ///
    /// * `type` - Message type, see [Types](#struct.Types)
    /// * `object` - A `usize` value
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct UsizeObject {
        pub r#type: String,
        pub object: usize,
    }

    impl GetMessageType for UsizeObject {
        /// Get message type or error
        ///
        /// # Example
        /// ```
        /// let msg: Result<StringObject, _> = serde_json::from_str(m);
        ///     match msg {
        ///         Ok(msg) => match msg.get_type() {
        ///             Ok(Types::Raised) => ()
        ///             _ => )_
        ///         }
        ///     }
        /// ```
        fn get_type(&self) -> Result<Types, InvalidMessageType> {
            Types::from_str(self.r#type.as_str())
        }
    }

    /// Inbound message skeleton: Vec objects
    ///
    /// * `type` - Message type, see [Types](#struct.Types)
    /// * `pollobject` - A `String` value
    /// * `polloptionobject` - A `String` value
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct HashMapObject {
        pub r#type: String,
        pub object: HashMap<String, String>,
    }

    impl GetMessageType for HashMapObject {
        /// Get message type or error
        ///
        /// # Example
        /// ```
        /// let msg: Result<StringObject, _> = serde_json::from_str(m);
        ///     match msg {
        ///         Ok(msg) => match msg.get_type() {
        ///             Ok(Types::Raised) => ()
        ///             _ => )_
        ///         }
        ///     }
        /// ```
        fn get_type(&self) -> Result<Types, InvalidMessageType> {
            Types::from_str(self.r#type.as_str())
        }
    }
}

pub mod outbound {
    use serde::Serialize;
    use serde_json::Value as Arbitrary;
    use std::collections::HashMap;
    use std::{error, fmt};

    use crate::server;
    /// Error if message has unknown message type
    ///
    /// For all known types, see
    /// [Types](#struct.Types)
    #[derive(Debug)]
    pub struct InvalidMessageType;

    impl fmt::Display for InvalidMessageType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Invalid message type")
        }
    }

    // see https://doc.rust-lang.org/stable/rust-by-example/error/multiple_error_types/define_error_type.html
    impl error::Error for InvalidMessageType {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            None
        }
    }

    /// Get type of any message struct
    ///
    /// Ensuring that every struct representing a message skeleton implements the same basic
    /// functions to return the message type
    pub trait GetMessageType {
        fn get_type(&self) -> Result<Types, InvalidMessageType>;
    }

    /// All known types of incoming messages
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Types {
        User,
        // {
        //     "type": "joined",
        //     "object" : {
        //         "name": user_name,
        //         "id": user_id,
        //         "elevated": elevated
        //     }
        All,
        // {
        //     "type": "all",
        //     "raised": room.raised,
        //     "joined": room.connected,
        // }
        SelfStatus, // r#Self is restricted https://internals.rust-lang.org/t/raw-identifiers-dont-work-for-all-identifiers/9094/3
        // {
        //     "type": "self",
        //     "id": user_id,
        // },
        Raised,
        // {
        //     "type": "raised",
        //     "owner_id": msg.owner_id,
        //     "owner_name": msg.owner_name,
        //     "object": &msg.object,
        //     "elevated": elevated,
        // }
        Lower,
        // {
        //     "type": "lower",
        //     "owner_id": msg.owner_id,
        //     "owner_name": msg.owner_name,
        //     "object": msg.object,
        //     "elevated": elevated,
        // });
        Instant,
        // {
        //     "type": "instant",
        //     "owner_id": msg.owner_id,
        //     "owner_name": msg.owner_name,
        //     "object": msg.object,
        //     "elevated": elevated,
        // }
        Elevated,
        // {
        //     "type": "elevated",
        //     "object": msg.object,
        // }
        Receded,
        // {
        //     "type": "receded",
        //     "object": msg.object,
        // }
        Error,
        // {
        //     "type": "error",
        //     "object": "error description",
        // }
        VoteDelete,
        // {
        //      "type": "deletevote",
        //      "pollobject": poll.title,
        //      "polloptionobject": poll_option_title,
        //      "userid": user_id, // or 0 in case of not elevated users
        // }
        Poll,
        // {
        //     "type": "poll",
        //     "object": "amazing poll title",
        // }
        PollOption,
        // {
        //     "type": "poll",
        //     "pollobject": "amazing poll title",
        //     "polloptionobject": "amazing poll-option title",
        // }
        Vote,
        // {
        //      "type": "vote",
        //      "pollobject": poll_title,
        //      "polloptionobject": poll_option_title,
        //      "username": vote.owner_name, // or 0 in case of not elevated user
        //      "userid": vote.owner_id, // or "" in case of not elevated user
        // }
        PollClose,
        // {
        //      "type": "closepoll",
        //      "pollobject": poll.title,
        // }
    }

    /// Message skeleton containing the current state of a room
    #[derive(Serialize)]
    pub struct All {
        pub r#type: Types,
        pub raised: Vec<server::Raised>,
        pub joined: HashMap<usize, server::User>,
    }

    #[derive(Serialize)]
    pub struct UserFormat {
        pub id: usize,
        pub name: String,
        pub elevated: bool,
    }

    /// Message skeleton containing the current state of a user
    #[derive(Serialize)]
    pub struct User {
        pub r#type: Types,
        pub object: UserFormat,
    }

    /// Message skeleton representing an object an its metadata
    /// # Parameters
    /// * `type` - Message type. Expected: Raised, Lowered, Instant
    /// * `owner_id` - Owner's user ID
    /// * `owner_name` - Owner's name
    /// * `object` - The represented object
    #[derive(Serialize)]
    pub struct OwnedObject {
        pub r#type: Types,
        pub owner_id: usize,
        pub owner_name: String,
        pub object: Arbitrary,
        pub elevated: bool,
    }

    /// Message skeleton to change a user's permissions
    /// # Parameters
    /// * `type` - Message type. Exprected: Elevated, Receded
    /// * `object` - Target user's ID
    #[derive(Serialize)]
    pub struct PermissionChange {
        pub r#type: Types,
        pub object: usize,
    }

    /// Message skeleton to send an error
    /// # Parameters
    /// * `type` - Message type. Exprected: Error
    /// * `object` - Error Code
    /// * `description` - Error Description
    #[derive(Serialize)]
    pub struct Error {
        pub r#type: Types,
        pub object: String,
        pub description: String,
    }

    /// Message skeleton to delete a user's vote
    /// # Parameters
    /// * `type` - Message type. Exprected: VoteDelete
    /// * `pollobject` - Title of the poll
    /// * `polloptionobject` - Title of the poll-option
    /// * `userid` - ID of the user (or 0 is the receiver is not elevated)
    #[derive(Serialize)]
    pub struct VoteDelete {
        pub r#type: Types,
        pub pollobject: String,
        pub polloptionobject: String,
        pub userid: usize,
    }

    // Message skeleton to send a poll
    /// # Parameters
    /// * `type` - Message type. Exprected: Poll
    /// * `object` - Title of the poll
    #[derive(Serialize)]
    pub struct Poll {
        pub r#type: Types,
        pub object: String,
    }

    // Message skeleton to send a poll-option
    /// # Parameters
    /// * `type` - Message type. Exprected: PollOption
    /// * `pollobject` - Title of the poll
    /// * `polloptionobject` - Title of the poll-option
    #[derive(Serialize)]
    pub struct PollOption {
        pub r#type: Types,
        pub pollobject: String,
        pub polloptionobject: String,
    }

    // Message skeleton to send a vote
    /// # Parameters
    /// * `type` - Message type. Exprected: Vote
    /// * `pollobject` - Title of the poll
    /// * `polloptionobject` - Title of the poll-option
    /// * `username` - Name of the voting-user (or "" if the receiver is not elevated)
    /// * `userid` - ID of the voting-user (or 0 if the receiver is not elevated)
    #[derive(Serialize)]
    pub struct Vote {
        pub r#type: Types,
        pub pollobject: String,
        pub polloptionobject: String,
        pub username: String,
        pub userid: usize,
    }

    // Message skeleton to close a poll
    /// # Parameters
    /// * `type` - Message type. Exprected: PollClose
    /// * `object` - Title of the poll
    #[derive(Serialize)]
    pub struct PollClose {
        pub r#type: Types,
        pub object: String,
    }
}
