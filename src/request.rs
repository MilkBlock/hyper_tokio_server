use std::{fmt::format, rc::Rc, sync::{Arc }};
use tokio::sync::Mutex;

use futures::stream::SplitSink;
use futures_util::{FutureExt, Sink, SinkExt, StreamExt};

use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::{Message, WebSocket}, WebSocketStream};

use crate::{debug_info, BoardInfo, BoardOrApp, ServerData, SinkArctex, StreamArctex, WsThreadInfo};

pub async fn register_app(ws_thread_arctex:Arc<Mutex<WsThreadInfo>>,server_data_arctex:Arc<Mutex<ServerData>>,write:SinkArctex,read:StreamArctex,
    user_name:&String , room_num:&i32){
    let mut server_data = server_data_arctex.lock().await;
    if !server_data.rooms.contains(room_num){
        // 不包含这个room则插入新的room
        server_data.rooms.push(*room_num);
    }
    ws_thread_arctex.lock().await.board_or_app = BoardOrApp::App;
    println!("register app user_name:{} room_num:{}",user_name,room_num)
}
pub async fn register_board(ws_thread_arctex:Arc<Mutex<WsThreadInfo>>,server_data_arc_mtx:Arc<Mutex<ServerData>>,write:SinkArctex,read:StreamArctex,
    wifi:&String , ip:&String){
    let mut server_data = server_data_arc_mtx.lock().await;
    let board_id = server_data.boards.insert( BoardInfo{
        wifi : wifi.clone(),ip : ip.clone(),write:write.clone(), read:read.clone()
    });
    debug_info!("{:?}",server_data.boards);
    // 对thread info
    ws_thread_arctex.lock().await.board_or_app = BoardOrApp::Board { board_id: board_id };
    println!("register board wifi:{} ip:{} board_or_app {:?}",wifi,ip,ws_thread_arctex.lock().await.board_or_app);
}

pub async fn request_list_rooms(ws_thread_info_arctex:Arc<Mutex<WsThreadInfo>>,server_data_arc_mtx:Arc<Mutex<ServerData>>,write:SinkArctex,read:StreamArctex){
    let (server_data,mut write)= (server_data_arc_mtx.lock().await,write.lock().await);
    // server_data.rooms.iter().map(|x| x.to_string()).
    let rooms_strs:Vec<String>  = server_data.rooms.iter().map(i32::to_string).collect();
    let rooms_str  =rooms_strs.join(",");
    write.send(Message::Text(format!("response_list_rooms:({})",rooms_str))).await;
    debug_info!("{}",Message::Text(format!("response_list_rooms:({})",rooms_str)).to_string());
}
pub async fn request_list_boards(ws_thread_info_arctex:Arc<Mutex<WsThreadInfo>>,server_data_arc_mtx:Arc<Mutex<ServerData>>,write:SinkArctex,read:StreamArctex){
    let (server_data,mut write)= (server_data_arc_mtx.lock().await,write.lock().await);
    // server_data.rooms.iter().map(|x| x.to_string()).
    let id_wifi_ip_tuple_strs:Vec<String>  = server_data.boards.iter()
        .map(|(board_id, board_info)|format!("({},{},{})",board_id,board_info.wifi,board_info.ip).to_string()).collect();
    let id_wifi_ip_str  =format!("{}",id_wifi_ip_tuple_strs.join(","));
    write.send(Message::Text(format!("response_list_boards:({})",id_wifi_ip_str))).await;
    debug_info!("{}",Message::Text(format!("response_list_boards:({})",id_wifi_ip_str)).to_string());
}

// deprecated 
pub async fn disconnect(ws_thread_info_arctex:Arc<Mutex<WsThreadInfo>>,server_data_arc_mtx:Arc<Mutex<ServerData>>,write:SinkArctex,read:StreamArctex){
    let (server_data,mut write)= (server_data_arc_mtx.lock(),write.lock().await);
    write.flush();
    write.send(Message::Close(None)).await;
}

pub async fn log(ws_thread_info:Arc<Mutex<WsThreadInfo>>,server_data_arc_mtx:Arc<Mutex<ServerData>>,write:SinkArctex,read:StreamArctex,log_string:&String){
    println!("\x1B[34m{}\x1B[0m",log_string)
}