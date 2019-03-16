use super::song::download_song;
use super::web_socket::MyWebSocket;
use actix::*;
pub struct MyIO;

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
    SaveSongToState { song_name: String },
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
                // boolean informing us whether song was downloaded or not
                let song_downloaded = download_song(&song_name);

                let (additional_action, message) = {
                    // if song was downloaded save it's name to the state
                    if song_downloaded {
                        (
                            AdditionalAction::SaveSongToState {
                                song_name: song_name.clone(),
                            },
                            "song_download_success".to_owned(),
                        )
                    // otherwise do nothing
                    } else {
                        (AdditionalAction::None, "song_download_failure".to_owned())
                    }
                };
                // construct io message that will tell MyWebSocket what to do next
                let io_response = IOResponse {
                    message,
                    success: song_downloaded,
                    additional_action,
                };
                msg.sender_address.do_send(io_response);
            }
        }
    }
}
