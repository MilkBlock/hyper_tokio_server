#[derive(Debug)]
pub enum Character{
    // 这个在 register board 中指定
    Board{
        board_id : usize,
        // op_room_num: Option<u32>
    },
    App{
        // room_num: u32,
        board_id: usize
    },
    Referee{
        // room_num: u32,
        referee_id: usize
    },
    Visitor{
        visitor_id: usize,
        // room_num: u32
    },
    NotSure,
}
