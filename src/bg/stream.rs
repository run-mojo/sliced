use actix::*;
use bg::Bg;
use redis::Redis;
use std::time::Duration;

pub struct Load;

pub struct LoadResult;

impl Message for Load {
    type Result = (i64);
}

impl Handler<Load> for Bg {
    type Result = i64;

    fn handle(&mut self, msg: Load, ctx: &mut Context<Self>) -> Self::Result {
        self.redis.run(|| {
            println!("hello redis event-loop!");
        });

        println!("hello stream::Load message");
        0
    }
}

#[derive(Message)]
pub struct Send;