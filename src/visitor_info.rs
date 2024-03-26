use crate::request::{SinkArctex, StreamArctex};
use std::fmt::Debug;

pub struct VisitorInfo{
    pub write: SinkArctex,
    pub read: StreamArctex,
    pub room_num: u32
}
impl Debug for VisitorInfo{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"VisitorInfo")
    }
}

impl VisitorInfo{
    pub fn new(read:StreamArctex,write:SinkArctex,room_num:u32)->Self{
        Self { write, read, room_num}
    }
}