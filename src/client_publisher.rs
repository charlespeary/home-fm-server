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
    type Result = ();
}

impl Handler<RegisterWS> for ClientPublisher {
    type Result = ();
    fn handle(&mut self, msg: RegisterWS, ctx: &mut Self::Context) -> Self::Result {
        // TODO : keep track of websockets that are not used anymore and delete them from the vector
        self.websockets.push(msg.addr);
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
