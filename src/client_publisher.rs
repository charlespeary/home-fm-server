use crate::song_queue::SongQueue;
use crate::web_socket::{MyWebSocket, UserMessage};
use actix::prelude::*;
use serde::Serialize;

#[derive(Default)]
/// Struct keeping track of connected websockets.
/// If some data needs to be sent to all of the clients, then it is forwarded through this struct.
pub struct ClientPublisher {
    websockets: Vec<Addr<MyWebSocket>>,
}

impl Actor for ClientPublisher {
    type Context = Context<Self>;
}

impl actix::Supervised for ClientPublisher {}

impl SystemService for ClientPublisher {
    fn service_started(&mut self, ctx: &mut Context<Self>) {
        println!("Service started");
    }
}

pub struct RegisterWS {
    pub addr: Addr<MyWebSocket>,
}

impl Message for RegisterWS {
    type Result = Result<(), ()>;
}

pub struct DeleteWS {
    pub ws_addr: Addr<MyWebSocket>,
}

impl Message for DeleteWS {
    type Result = ();
}

/// Deletes inactive websocket from the vector of available connections.
impl Handler<DeleteWS> for ClientPublisher {
    type Result = ();
    fn handle(&mut self, msg: DeleteWS, ctx: &mut Self::Context) -> Self::Result {
        let mut index = 0;
        let mut ws_found = false;
        for (i, addr) in self.websockets.iter().enumerate() {
            if addr.eq(&msg.ws_addr) {
                index = i;
                ws_found = true;
            }
        }

        if !ws_found {
            return;
        }
        self.websockets.remove(index);
    }
}

/// Adds new websocket's address to the vector of available connections.
impl Handler<RegisterWS> for ClientPublisher {
    type Result = Result<(), ()>;
    fn handle(&mut self, msg: RegisterWS, ctx: &mut Self::Context) -> Self::Result {
        self.websockets.push(msg.addr);
        Ok(())
    }
}

/// Sends message to every client in the vector of available connections.
impl<T> Handler<UserMessage<T>> for ClientPublisher
where
    T: Serialize + Send + 'static + Clone,
{
    type Result = ();
    fn handle(&mut self, msg: UserMessage<T>, ctx: &mut Self::Context) -> Self::Result {
        for ws in self.websockets.iter() {
            ws.do_send(msg.clone());
        }
    }
}
