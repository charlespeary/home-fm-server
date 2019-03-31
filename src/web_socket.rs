use self::actix::*;
use crate::client_publisher::{ClientPublisher, RegisterWS};
use crate::song::SongRequest;
use crate::song_queue::QueueJob;
use crate::system::AppState;
use actix_web::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// do websocket handshake and start `MyWebSocket` actor
pub fn ws_index(r: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    ws::start(r, MyWebSocket {})
}

#[derive(Debug, Default)]
pub struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // get ClientPublisher address and send address of websocket to it
        let publisher_addr = ClientPublisher::from_registry();
        publisher_addr.do_send(RegisterWS {
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
                println!("Sending message : {}", message);
                ctx.text(&message);
            }
            Err(e) => {
                eprintln!("Couldn't serialize given entity: {}", e);
            }
        }
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

#[derive(Serialize, Deserialize)]
struct EmptyValue {}

/// Handler for ws::Message message
impl StreamHandler<ws::Message, ws::ProtocolError> for MyWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                let request: Request = serde_json::from_str(&text).unwrap();
                match request.action.as_str() {
                    "request_song" => {
                        let song: Payload<SongRequest> = serde_json::from_str(&text).unwrap();
                        println!("requesting song");
                        ctx.state().queue_handler.do_send(QueueJob::DownloadSong {
                            requested_song: song.payload,
                        });

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

impl<T> Handler<UserMessage<T>> for MyWebSocket
where
    T: Serialize,
{
    type Result = ();
    fn handle(&mut self, msg: UserMessage<T>, ctx: &mut Self::Context) -> Self::Result {
        self.send_message(ctx, &msg);
    }
}
