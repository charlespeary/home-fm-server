use self::actix::*;
use super::io::*;
use super::io::{AdditionalAction, IOJob, IOResponse};
use super::radio::{RadioJob, RadioResponse};
use super::song::Song;
use crate::client_publisher::{ClientPublisher, RegisterWS};
use actix_web::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// do websocket handshake and start `MyWebSocket` actor
pub fn ws_index(r: &HttpRequest) -> Result<HttpResponse, Error> {
    ws::start(r, MyWebSocket {})
}

#[derive(Debug, Default)]
pub struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // get ClientPublisher address and send address of websocket to it
        let publisher_addr = ClientPublisher::from_registry();
        publisher_addr.send(RegisterWS {
            addr: ctx.address(),
        });
    }
}

const FIVE_SECONDS: Duration = Duration::from_secs(5);

impl MyWebSocket {
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
}

fn send_next_song(ctx: &Addr<MyWebSocket>, song: &Song) {
    let response = UserMessage::<Song> {
        success: true,
        action: "next_song".to_owned(),
        value: song.clone(),
    };
    ctx.send(response);
}

/// struct with data describing action that comes from client
#[derive(Serialize, Deserialize, Debug)]
struct WSAction {
    action: String,
    payload: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserMessage<T> {
    pub success: bool,
    pub action: String,
    pub value: T,
}

impl<T> Message for UserMessage<T> {
    type Result = ();
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
                        println!("{}", action_object.payload);
                        //                        self.IO.do_send(IOMessage {
                        //                            action: IOJob::DownloadSong {
                        //                                song_name: action_object.payload,
                        //                            },
                        //                            sender_address: ctx.address(),
                        //                        });
                        let response = UserMessage::<EmptyValue> {
                            success: true,
                            action: "start_song_download".to_owned(),
                            value: EmptyValue {},
                        };
                        self.send_message(ctx, &response);
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
                //                ctx.state().songs_queue.lock().unwrap().push(song.clone());
                //                let response = UserMessage::<Song> {
                //                    success: msg.success,
                //                    action: msg.message,
                //                    value: song.clone(),
                //                };
                //                self.send_message(ctx, &response);
            }
            _ => {
                let response = UserMessage::<EmptyValue> {
                    success: false,
                    action: msg.message,
                    value: EmptyValue {},
                };
                self.send_message(ctx, &response);
            }
        };
        ()
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
