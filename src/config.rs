use super::radio::{GetFrequency, SetFrequency};
use super::responses::get_standard_success_response;
use super::system::AppState;
use actix_web::{AsyncResponder, Error as AWError, FutureResponse, HttpResponse, Json, State};
use futures::future::Future;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
/// Configuration used mostly for radio commands.
pub struct Config {
    pub frequency: f32,
}

/// PUT /config
pub fn update_config(config: Json<Config>, state: State<AppState>) -> FutureResponse<HttpResponse> {
    state
        .radio
        .send(SetFrequency {
            frequency: config.frequency,
        })
        .and_then(|_| Ok(HttpResponse::Ok().json(get_standard_success_response())))
        .from_err()
        .responder()
}

/// GET /config
pub fn get_config(state: State<AppState>) -> FutureResponse<HttpResponse> {
    state
        .radio
        .send(GetFrequency {})
        .and_then(|frequency| Ok(HttpResponse::Ok().json(Config { frequency })))
        .from_err()
        .responder()
}
