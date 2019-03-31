use actix::*;

#[derive(Default)]
pub struct Radio;

impl Actor for Radio {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        //     self.next_song(ctx);
    }
}
