use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StandardResponse {
    success: bool,
}

pub fn get_standard_success_response() -> StandardResponse {
    StandardResponse { success: true }
}

pub fn get_standard_failure_response() -> StandardResponse {
    StandardResponse { success: false }
}
