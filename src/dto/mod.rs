use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct MessageRes {
    #[validate(length(min = 1, message = "Message cannot be empty"), required())]
    pub message: Option<String>,
}
