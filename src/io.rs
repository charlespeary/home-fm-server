use super::song::{download_song, Song};
use super::web_socket::MyWebSocket;
use crate::db::DBExecutor;
use actix::*;

pub struct MyIO {
    pub db: Addr<DBExecutor>,
}

#[derive(Debug)]
pub enum IOJob {
    DownloadSong { song_name: String },
}

impl Actor for MyIO {
    type Context = SyncContext<Self>;
}

#[derive(Debug, Message)]
pub struct IOMessage {
    pub action: IOJob,
    pub sender_address: Addr<MyWebSocket>,
}

#[derive(Debug)]
pub enum AdditionalAction {
    ScheduleSong { song: Song },
    None,
}

#[derive(Debug, Message)]
pub struct IOResponse {
    pub message: String,
    pub success: bool,
    pub additional_action: AdditionalAction,
}

impl Handler<IOMessage> for MyIO {
    type Result = ();

    fn handle(&mut self, msg: IOMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg.action {
            IOJob::DownloadSong { song_name } => {
                // Result containing Song with all informations of it we need or empty error for now
                let song = download_song(&song_name);
                let (additional_action, message, success) = {
                    // if song was downloaded save it's name to the state
                    match song {
                        Ok(song) => (
                            AdditionalAction::ScheduleSong { song },
                            "song_download_success".to_owned(),
                            true,
                        ),
                        _ => (
                            AdditionalAction::None,
                            "song_download_failure".to_owned(),
                            false,
                        ),
                    }
                };
                // construct io message that will tell MyWebSocket what to do next
                let io_response = IOResponse {
                    message,
                    success,
                    additional_action,
                };
                msg.sender_address.do_send(io_response);
            }
        }
    }
}
