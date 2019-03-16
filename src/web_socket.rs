use super::io::*;
use super::io::{AdditionalAction, IOJob, IOResponse};
use super::system::AppState;
use ::actix::*;
use actix_web::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
/// do websocket handshake and start `MyWebSocket` actor
pub fn ws_index(r: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    println!("Websocket spawned");
    ws::start(r, MyWebSocket {})
}

#[derive(Debug)]
pub struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.broadcast_state(ctx);
    }
}

const ONE_SECOND: Duration = Duration::from_secs(1);

impl MyWebSocket {
    fn broadcast_state(&self, ctx: &mut <Self as Actor>::Context) {
        // this interval is not needed at this moment
        // ctx.run_interval(ONE_SECOND, |act, ctx| {
        //     let x = ctx.state().current_song.lock().unwrap().clone();
        //     println!("I am running");
        //     ctx.text(&x);
        //     // ctx.text("hello");
        // });
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
    #[serde(rename = "unkown_action")]
    UnknownAction,
    #[serde(rename = "successfull_request")]
    Success,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyResponse<T> {
    pub success: bool,
    pub message: T,
}

/// Handler for ws::Message message
impl StreamHandler<ws::Message, ws::ProtocolError> for MyWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                let action_object: WSAction = serde_json::from_str(&text).unwrap();
                match action_object.action.as_str() {
                    "request_song" => {
                        // download song and set it to active, in case of any errors notify client about it
                        //     set_current_song(&action_object.payload.as_str(), ctx);
                        let address = ctx.address();
                        ctx.state().IO.do_send(IOMessage {
                            action: IOJob::DownloadSong {
                                song_name: action_object.payload,
                            },
                            sender_address: address,
                        });
                        ctx.text("Started downloading song.")
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

impl Handler<IOResponse> for MyWebSocket {
    type Result = ();
    fn handle(&mut self, msg: IOResponse, ctx: &mut Self::Context) -> Self::Result {
        match msg.additional_action {
            AdditionalAction::SaveSongToState { song_name } => {
                *(ctx.state().current_song.lock().unwrap()) = song_name;
            }
            _ => (),
        };

        let response = MyResponse {
            success: msg.success,
            message: msg.message,
        };

        ctx.text(serde_json::to_string(&response).unwrap());
    }
}
