use crate::web_socket::{MyWebSocket, UserMessage};
use actix::prelude::*;
use serde::Serialize;
#[derive(Default)]
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

impl Handler<DeleteWS> for ClientPublisher {
    // we return index of the websocket in the vector
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
        println!("Deleting ws at index - {}", index);
        self.websockets.remove(index);
    }
}

impl Handler<RegisterWS> for ClientPublisher {
    // we return index of the websocket in the vector
    type Result = Result<(), ()>;
    fn handle(&mut self, msg: RegisterWS, ctx: &mut Self::Context) -> Self::Result {
        println!("Registering new WS");
        self.websockets.push(msg.addr);
        Ok(())
    }
}

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
