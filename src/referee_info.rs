pub struct RefereeInfo{
    room_num: u32
}
impl RefereeInfo{
    pub fn new(room_num:u32)-> Self{
        Self{
            room_num
        }
    }
}