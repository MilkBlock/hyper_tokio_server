use std::{fmt::format, sync::Arc};
use tokio::sync::Mutex;

use futures::stream::{SplitSink, SplitStream};
use futures_util::SinkExt;

use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::{handshake::server, Message}, WebSocketStream};

use crate::{debug_info_blue, debug_info_green, debug_info_red, referee_info::RefereeInfo, room::{self, Room}, visitor_info::VisitorInfo, BoardInfo, Character, ServerData, WsThreadInfo};

pub type Sink = SplitSink<WebSocketStream<TcpStream>,Message>;
pub type Stream = SplitStream<WebSocketStream<TcpStream>>;
pub type SinkArctex = Arc<Mutex<Sink>>;
pub type StreamArctex = Arc<Mutex<Stream>>;

#[derive(Clone)]
pub struct Context{
    pub ws_thread_info_arctex:Arc<Mutex<WsThreadInfo>>,pub server_data_arctex:Arc<Mutex<ServerData>>,pub write:SinkArctex,pub read:StreamArctex
}

pub async fn register_app(ctx:Context,user_name:&String, room_num:&u32, board_idx:&usize){ 
    
    debug_info_green!("start register app");
    let mut server_data = ctx.server_data_arctex.lock().await;
    ctx.ws_thread_info_arctex.lock().await.character = Character::App {  board_id: *board_idx, };
    let mut flag = false; 
    for room in server_data.rooms.iter_mut(){
        if room_num == &room.room_num{
            if room.board_nums.contains(board_idx){
                debug_info_red!("room {} 已经包含了 board {}",room_num,board_idx);
            }else {
                room.board_nums.push(*board_idx);
                break;
            }
            flag = true;
        }
    }
    // 如果没有这个room
    if flag == false{
        let mut room = Room::new(*room_num);
        room.board_nums.push(*board_idx);
        server_data.rooms.push(room);
    }

    // 并把 app 对应的board 的 board info 的room_num 也设置成这个room_num
    match server_data.boards.get_mut(*board_idx){
        Some(board) => {
            if !board.op_room_num.is_none(){
                debug_info_red!("这个board 已经进入了别的房间");
            }
            board.op_room_num = Some(*room_num) 
        },
        None => debug_info_red!("用这个board id {} 找不到 Board",board_idx),
    }
    println!("register app user_name:{} room_num:{}",user_name,room_num)
}
pub async fn register_board(ctx:Context, wifi:&String , ip:&String){
    let mut server_data = ctx.server_data_arctex.lock().await;
    let board_id = server_data.boards.insert( BoardInfo::new(wifi.clone(), ip.clone(), ctx.read.clone(),ctx.write.clone()));
    debug_info_green!("{:?}",server_data.boards);
    // 对thread info
    ctx.ws_thread_info_arctex.lock().await.character = Character::Board { board_id: board_id};
    println!("register board wifi:{} ip:{} board_or_app {:?}",wifi,ip,ctx.ws_thread_info_arctex.lock().await.character);
}

pub async fn request_list_rooms(ctx:Context){
    let (server_data,mut write)= (ctx.server_data_arctex.lock().await,ctx.write.lock().await);
    // server_data.rooms.iter().map(|x| x.to_string()).
    let rooms_strs:Vec<String> = server_data.rooms.iter().map(|room|room.room_num.to_string()).collect();
    let rooms_str  =rooms_strs.join(",");
    write.send(Message::Text(format!("response_list_rooms:({})",rooms_str))).await;
    debug_info_blue!("{}",Message::Text(format!("response_list_rooms:({})",rooms_str)).to_string());
}
pub async fn request_list_boards(ctx:Context){
    let (server_data,mut write)= (ctx.server_data_arctex.lock().await,ctx.write.lock().await);
    // server_data.rooms.iter().map(|x| x.to_string()).
    let id_wifi_ip_tuple_strs:Vec<String>  = server_data.boards.iter()
        .map(|(board_id, board_info)|format!("({},{},{})",board_id,board_info.wifi,board_info.ip).to_string()).collect();
    let id_wifi_ip_str  =format!("{}",id_wifi_ip_tuple_strs.join(","));
    write.send(Message::Text(format!("response_list_boards:({})",id_wifi_ip_str))).await;
    debug_info_blue!("{}",Message::Text(format!("response_list_boards:({})",id_wifi_ip_str)).to_string());
}

pub async fn request_get_model_type(ctx:Context){
    let (server_data,mut write)= (ctx.server_data_arctex.lock().await,ctx.write.lock().await);
    // server_data.rooms.iter().map(|x| x.to_string()).
    write.send(Message::Text(format!("response_get_model_type:({})",1))).await;
    debug_info_red!("{}",Message::Text(format!("send:({})",1)).to_string());
}

// deprecated 
pub async fn log(ctx:Context,log_string:&String){
    println!("\x1B[34m{}\x1B[0m",log_string)
}

// Referee end
pub async fn register_referee(ctx:Context,room_num:&u32){ 
    let mut server_data = ctx.server_data_arctex.lock().await;
    let referee_index = server_data.referees.insert(RefereeInfo::new(*room_num));
    ctx.ws_thread_info_arctex.lock().await.character = Character::Referee {  referee_id: referee_index};
    println!("register referee in room_num:{}",room_num)
}
pub async fn register_visitor(ctx:Context,room_num:&u32){ 
    let mut server_data = ctx.server_data_arctex.lock().await;
    let visitor_id = server_data.visitors.insert(VisitorInfo::new(ctx.read.clone(), ctx.write.clone(),*room_num));
    ctx.ws_thread_info_arctex.lock().await.character = Character::Visitor { visitor_id };
    println!("register visitor as visitor_id:{} in room_num:{}",visitor_id,room_num)
}

// pub async fn request_set_
pub async fn request_set_board_coords(ctx:Context,board_id:&usize,x:&f32,y:&f32) {
    debug_info_green!("send x:{} y:{} to board {}",x,y,board_id);
    let mut server_data = ctx.server_data_arctex.lock().await;
    let (msg,board_op_room_num) = match server_data.boards.get(*board_id){
        Some(board_info) => {
            (Message::Text(format!("request_set_board_coords:({},{},{})",board_id,x,y)),board_info.op_room_num.clone())
        },
        None => {debug_info_red!("未找到这个 board_id:{} 对应的 board",board_id);return;},
    };
    for (_,visitor) in server_data.visitors.iter_mut(){
        if Some(visitor.room_num) == board_op_room_num{
            visitor.write.lock().await.send(msg.clone());
        }else{
            debug_info_red!("未找到这个 board_id:{} 对应的 board",board_id);return;
        }
    }
}
pub async fn request_list_boards_in_room(ctx:Context,room_num:&u32){
    let (server_data,mut write)= (ctx.server_data_arctex.lock().await,ctx.write.lock().await);
    let board_ids_names:Vec<String>  = server_data.boards.iter().filter(|(board_id, board_info)|board_info.op_board_name.is_some()&& board_info.op_room_num.unwrap() == *room_num)
            .map(|(board_id, board_info)| format!("({},{})",board_id,board_info.op_board_name.clone().unwrap_or_else(||{debug_info_red!("这个 board_id:{} 没有对应的 board_name",board_id);"no_name".to_string()})))
            .collect();
    let str_board_ids_names: String  =format!("{}",board_ids_names.join(","));
    write.send(Message::Text(format!("response_list_boards:({})",str_board_ids_names))).await;
    debug_info_blue!("{}",Message::Text(format!("response_list_boards:({})",str_board_ids_names)).to_string());
}

// visitor related 
// pub async fn request_get_board_coords(ctx:Context,board_id:&usize) {
//     let mut server_data = ctx.server_data_arctex.lock().await;
//     let op_board_info = server_data.boards.get_mut(*board_id);
//     match op_board_info{
//         Some(board_info) => {
//             debug_info_green!("send x:{} y:{} to visitor {}",board_info.pos_x,board_info.pos_y,board_id);
//             let msg = Message::Text(format!("response_set_baord_coords:({},{},{})",board_id,board_info.pos_x,board_info.pos_y));
//             visito.write.lock().await.send(msg).await;
//         },
//         None => debug_info_green!("未找到这个 board_id:{} 对应的 board",board_id),
//     }
// }
pub async fn check(ctx:Context){
    match ctx.ws_thread_info_arctex.lock().await.character{
        Character::Board { board_id } => println!("board {} checked",board_id),
        Character::App { board_id } => println!("app of board {} checked",board_id),
        Character::Referee { referee_id } => println!("referee {} checked",referee_id),
        Character::Visitor { visitor_id } => println!("visitor {} checked",visitor_id),
        Character::NotSure => {
            // do nothing 
        },
        Character::Exited => debug_info_red!("收到了来自一个已经断开连接的客户端的 在线消息 "),
    }
}