use super::song::download_song;
use super::web_socket::{MyResponse, MyWebSocket};
use actix::*;
use failure::Error;
pub struct MyIO;

#[derive(Debug)]
pub enum IOJob {
    DownloadSong { song_name: String },
}

impl Actor for MyIO {
    type Context = SyncContext<Self>;
}

#[derive(Debug)]
pub struct IOMessage {
    pub action: IOJob,
    pub sender_address: Addr<MyWebSocket>,
}

impl Message for IOMessage {
    type Result = Result<(), Error>;
}

impl Handler<IOMessage> for MyIO {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: IOMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg.action {
            IOJob::DownloadSong { song_name } => {
                let download_status = download_song(&song_name);
                let message = MyResponse {
                    success: download_status.is_success(),
                    message: download_status.get_status(),
                };
                msg.sender_address.do_send(message);
            }
        }
        Ok(())
    }
}
