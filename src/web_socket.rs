use self::actix::*;
use crate::client_publisher::{ClientPublisher, DeleteWS, RegisterWS};
use crate::song::SongRequest;
use crate::song_queue::{BroadcastState, QueueJob};
use crate::system::AppState;
use actix_web::*;
use futures::future::Future;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// do websocket handshake and start `MyWebSocket` actor
pub fn ws_index(r: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    ws::start(r, MyWebSocket::new())
}

#[derive(Debug)]
pub struct MyWebSocket {
    hb: Instant,
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // get ClientPublisher address and send address of websocket to it
        self.hb(ctx);
        let publisher_addr = ClientPublisher::from_registry();
        publisher_addr.do_send(RegisterWS {
            addr: ctx.address(),
        });
        ctx.state().queue_handler.do_send(BroadcastState {});
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("stopping");
        ClientPublisher::from_registry().do_send(DeleteWS {
            ws_addr: ctx.address(),
        });
    }
}

const FIVE_SECONDS: Duration = Duration::from_secs(5);

impl MyWebSocket {
    pub fn new() -> Self {
        MyWebSocket { hb: Instant::now() }
    }

    fn send_message<T>(&self, ctx: &mut <Self as Actor>::Context, msg: &UserMessage<T>)
    where
        T: Serialize,
    {
        // serialize message to string in order to be able to send it
        match serde_json::to_string(msg) {
            Ok(message) => {
                ctx.text(&message);
            }
            Err(e) => {
                eprintln!("Couldn't serialize given entity: {}", e);
            }
        }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
}

/// struct with data describing action that comes from client
#[derive(Serialize, Deserialize, Debug)]
struct Request {
    action: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserMessage<T> {
    pub success: bool,
    pub action: String,
    pub value: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload<T> {
    payload: T,
}

impl<T> Message for UserMessage<T> {
    type Result = ();
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EmptyValue {}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeleteSongFromQueue {
    uuid: Uuid,
}

/// Handler for ws::Message message
impl StreamHandler<ws::Message, ws::ProtocolError> for MyWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let request: Request = serde_json::from_str(&text).unwrap();
                match request.action.as_str() {
                    "request_song" => {
                        let song = serde_json::from_str::<Payload<SongRequest>>(&text);
                        let response = if let Ok(song) = song {
                            ctx.state().queue_handler.do_send(QueueJob::DownloadSong {
                                requested_song: song.payload,
                            });
                            UserMessage::<EmptyValue> {
                                success: true,
                                action: "start_song_download".to_owned(),
                                value: EmptyValue {},
                            }
                        } else {
                            UserMessage::<EmptyValue> {
                                success: true,
                                action: "incomplete_data".to_owned(),
                                value: EmptyValue {},
                            }
                        };
                        self.send_message(ctx, &response);
                    }
                    "skip_song" => {
                        ctx.state().queue_handler.do_send(QueueJob::SkipSong {});
                    }
                    "delete_song_from_queue" => {
                        let song_uuid = serde_json::from_str::<Payload<DeleteSongFromQueue>>(&text);
                        if let Ok(song_uuid) = song_uuid {
                            ctx.state()
                                .queue_handler
                                .do_send(QueueJob::DeleteSongFromQueue {
                                    uuid: song_uuid.payload.uuid,
                                });
                        } else {
                            // data is not complete, send error message
                            let response = UserMessage::<EmptyValue> {
                                success: true,
                                action: "incomplete_data".to_owned(),
                                value: EmptyValue {},
                            };
                            self.send_message(ctx, &response);
                        }
                    }
                    _ => {
                        // Unkown action, let's notify user about that
                        let response = UserMessage::<EmptyValue> {
                            success: false,
                            action: "unknown_action".to_owned(),
                            value: EmptyValue {},
                        };
                        self.send_message(ctx, &response);
                    }
                }
                ctx.text(text)
            }
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl<T> Handler<UserMessage<T>> for MyWebSocket
where
    T: Serialize,
{
    type Result = ();
    fn handle(&mut self, msg: UserMessage<T>, ctx: &mut Self::Context) -> Self::Result {
        self.send_message(ctx, &msg);
    }
}
