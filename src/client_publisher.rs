use crate::web_socket::MyWebSocket;
use actix::prelude::*;

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
        println!("got message from some actor");
    }
}
