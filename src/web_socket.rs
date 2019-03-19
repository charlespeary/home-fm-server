use super::io::*;
use super::io::{AdditionalAction, IOJob, IOResponse};
use super::radio::{RadioJob, RadioResponse};
use super::song::{get_random_song, Song};
use super::system::AppState;
use ::actix::*;
use actix_web::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
/// do websocket handshake and start `MyWebSocket` actor
pub fn ws_index(r: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    ws::start(r, MyWebSocket {})
}

#[derive(Debug)]
pub struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.next_song(ctx);
    }
}

const FIVE_SECONDS: Duration = Duration::from_secs(5);

impl MyWebSocket {
    fn next_song(&self, ctx: &mut <Self as Actor>::Context) {
        let mut songs_queue = ctx.state().songs_queue.lock().unwrap();
        let next_song = if let Some(song) = songs_queue.first() {
            let song = song.clone();
            songs_queue.remove(0);
            song
        } else {
            get_random_song()
        };
        drop(songs_queue);

        ctx.state().radio.do_send(RadioJob::PlaySong {
            song: next_song.clone(),
            ws_addr: ctx.address(),
        });

        let response = MyResponse::<Song> {
            success: true,
            action: "next_song".to_owned(),
            value: next_song,
        };

        ctx.text(serde_json::to_string(&response).unwrap());
    }
}

/// struct with data describing action that comes from client
#[derive(Serialize, Deserialize, Debug)]
struct WSAction {
    action: String,
    payload: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyResponse<T> {
    pub success: bool,
    pub action: String,
    pub value: T,
}
#[derive(Serialize, Deserialize)]
struct EmptyValue {}

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
                        println!("{}", action_object.payload);
                        ctx.state().IO.do_send(IOMessage {
                            action: IOJob::DownloadSong {
                                song_name: action_object.payload,
                            },
                            sender_address: address,
                        });
                        ctx.text("Started downloading song.")
                    }
                    _ => {
                        // Unkown action, let's notify user about that
                        let response = MyResponse::<EmptyValue> {
                            success: false,
                            action: "unknown_action".to_owned(),
                            value: EmptyValue {},
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
            AdditionalAction::ScheduleSong { song } => {
                ctx.state().songs_queue.lock().unwrap().push(song.clone());
                let response = MyResponse::<Song> {
                    success: msg.success,
                    action: msg.message,
                    value: song.clone(),
                };
                ctx.text(serde_json::to_string(&response).unwrap());
            }
            _ => {
                let response = MyResponse::<EmptyValue> {
                    success: false,
                    action: msg.message,
                    value: EmptyValue {},
                };
                ctx.text(serde_json::to_string(&response).unwrap());
            }
        };
    }
}

impl Handler<RadioResponse> for MyWebSocket {
    type Result = ();
    fn handle(&mut self, msg: RadioResponse, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            RadioResponse::NextSong => {
                self.next_song(ctx);
            }
        }
    }
}
