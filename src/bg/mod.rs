//use actix::*;
//use redis::Redis;
//use std::time::Duration;
//
//pub mod stream;
//
//pub struct Bg {
//    pub redis: Redis
//}
//
//impl Actor for Bg {
//    type Context = Context<Self>;
//
//    fn started(&mut self, ctx: &mut Self::Context) {
//        println!("started background actor");
////        System::current().stop();
//    }
//}
