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
    type Result = Result<usize, ()>;
}

pub struct DeleteWS {
    pub index: usize,
}

impl Message for DeleteWS {
    type Result = ();
}

impl Handler<DeleteWS> for ClientPublisher {
    // we return index of the websocket in the vector
    type Result = ();
    fn handle(&mut self, msg: DeleteWS, ctx: &mut Self::Context) -> Self::Result {
        println!("{} {} ", self.websockets.len(), msg.index);
        if self.websockets.len() < msg.index {
            return;
        }
        self.websockets.remove(msg.index);
        println!(
            "Deleted ws at index - {}, length of vector after deletion - {} ",
            msg.index,
            self.websockets.len()
        );
    }
}

impl Handler<RegisterWS> for ClientPublisher {
    // we return index of the websocket in the vector
    type Result = Result<usize, ()>;
    fn handle(&mut self, msg: RegisterWS, ctx: &mut Self::Context) -> Self::Result {
        // TODO : keep track of websockets that are not used anymore and delete them from the vector
        self.websockets.push(msg.addr);
        // minus 1, because we want to get index which is e.g 0 not 1 (we're not in Lua!!!)
        Ok(self.websockets.len() - 1)
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

//pub struct RawMessage<'a> {
//    message: &'a str,
//}
//
//impl<'a> Message for RawMessage<'a> {
//    type Result = ();
//}
//
//impl<'a> Handler<RawMessage<'a>> for ClientPublisher
//{
//    type Result = ();
//    fn handle(&mut self, msg: RawMessage, ctx: &mut Self::Context) -> Self::Result {
//
//        for ws in self.websockets.iter() {
//            ws.do_send();
//        }
//    }
//}
