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
        Joined,
        All,
        SelfStatus, // r#Self is restricted https://internals.rust-lang.org/t/raw-identifiers-dont-work-for-all-identifiers/9094/3
        Raised,
        Lower,
        Instant,
        Elevated,
        Receded,
    }

    /// Message skeleton containing the current state of a room
    #[derive(Serialize)]
    pub struct All {
        pub r#type: Types,
        pub raised: Vec<server::Raised>,
        pub joined: HashMap<usize, server::User>,
    }

    /// Representing a user within a message to stay consisting
    /// across messages
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
        pub elevated: bool,
    }
}
