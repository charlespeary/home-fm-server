use super::song::{set_current_song, DownloadStatus};
use super::system::AppState;
use ::actix::*;
use actix_web::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// do websocket handshake and start `MyWebSocket` actor
pub fn ws_index(r: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    ws::start(r, MyWebSocket {})
}

struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.broadcast_state(ctx);
    }
}

const ONE_SECOND: Duration = Duration::from_secs(1);

// TODO: Broadcast is blocked while downloading song
impl MyWebSocket {
    fn broadcast_state(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(ONE_SECOND, |act, ctx| {
            let x = ctx.state().current_song.lock().unwrap().clone();
            println!("I am running");
            ctx.text(&x);
            // ctx.text("hello");
        });
    }
}

/// struct with data describing action that comes from client
#[derive(Serialize, Deserialize, Debug)]
struct WSAction {
    action: String,
    payload: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum Status {
    #[serde(rename = "Unkown action")]
    UnknownAction,
    #[serde(rename = "Successfull request")]
    Success,
}

#[derive(Serialize, Deserialize, Debug)]
struct MyResponse<T> {
    success: bool,
    message: T,
}

/// Handler for ws::Message message
impl StreamHandler<ws::Message, ws::ProtocolError> for MyWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                let action_object: WSAction = serde_json::from_str(&text).unwrap();
                match action_object.action.as_str() {
                    "set_current_song" => {
                        // download song and set it to active, in case of any errors notify client about it
                        let download_response =
                            set_current_song(&action_object.payload, ctx.state());
                        // response that gets serialized into json response
                        let response = MyResponse::<DownloadStatus> {
                            success: download_response.is_success(),
                            message: download_response.get_status(),
                        };
                        // send json via websocket to client after serialization
                        ctx.text(serde_json::to_string(&response).unwrap())
                    }
                    "schedule_song" => {}
                    _ => {
                        // Unkown action, let's notify user about that
                        let response = MyResponse::<Status> {
                            success: false,
                            message: Status::UnknownAction,
                        };
                        ctx.text(serde_json::to_string(&response).unwrap())
                    }
                }
                ctx.text(text)
            }
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}
