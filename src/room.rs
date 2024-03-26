
pub struct Room{
    pub room_num:u32,
    pub board_nums:Vec<u32>,
}

impl PartialEq for Room{
    fn eq(&self, other: &Self) -> bool {
        self.room_num == other.room_num
    }
    
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl PartialOrd for Room{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.room_num.partial_cmp(&other.room_num)
    }
}

impl Room{
    pub fn new(room_num:u32)->Room{
        Room{
            room_num,
            board_nums: vec![],
        }
    }
}
impl Eq for Room{

}
impl Ord for Room{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.room_num.cmp(&other.room_num)
    }
}