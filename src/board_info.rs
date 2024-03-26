use crate::{character::Character, request::{SinkArctex, StreamArctex}};
use std::fmt::Debug;

#[derive(Clone)]
pub struct BoardInfo{
    pub wifi:String,
    pub ip:String,
    pub write: SinkArctex,
    pub read: StreamArctex,
    pub pos_x: f32,
    pub pos_y: f32,
    pub op_room_num: Option<u32>,
    pub op_board_name: Option<String>,
}
impl Debug for BoardInfo{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoardInfo").field("wifi", &self.wifi).field("ip", &self.ip).finish()
    }
}

impl BoardInfo{
    pub fn new(wifi:String,ip:String,read:StreamArctex,write:SinkArctex)->Self{
        // 如果传入的 char 不是  board_char 就报错
        BoardInfo{
            wifi,
            ip,
            write,
            read,
            pos_x: 0.0,
            pos_y: 0.0,
            op_room_num: None,
            op_board_name: None,
        }
    }
}