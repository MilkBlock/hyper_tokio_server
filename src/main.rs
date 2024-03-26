use board_info::BoardInfo;
use character::Character;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};

use referee_info::RefereeInfo;
use request::{SinkArctex, StreamArctex};
use room::Room;
use slab::Slab;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};

mod request;
mod command_args;
mod macros;
mod room;
mod character;
mod board_info;
mod visitor_info;
mod referee_info;
use tokio::sync::Mutex;
use visitor_info::VisitorInfo;
use std::sync::{Arc};
use std::fmt::Debug;
use std::time::Duration;

use crate::command_args::MessageArgs;
use crate::request::{ log, register_app, register_board, register_referee, register_visitor, request_get_model_type, request_list_boards, request_list_boards_in_room, request_list_rooms, request_set_board_coords, Context};
pub struct ServerData{
    rooms : Vec<Room>,
    boards : Slab<BoardInfo>,
    visitors : Slab<VisitorInfo>,
    referees : Slab<RefereeInfo>
}
pub struct WsThreadInfo{
    character :Character,
    connect_id : u32,
}
impl WsThreadInfo{
    pub fn new(connect_id:u32) -> Self{
        WsThreadInfo{
            character: Character::NotSure,
            connect_id
        }
    }
}


/// A WebSocket server
#[tokio::main]
async fn main () {
    let server = TcpListener::bind("0.0.0.0:11451").await.expect("bind 端口失败");
    // ? 服务端端口注册完毕，进行数据初始化
    let mut _server_data = ServerData{rooms:Vec::from([Room::new(1)]),boards:Slab::new(),visitors:Slab::new(), referees: Slab::new() };
    let server_data_arctex =Arc::new(Mutex::new(_server_data));
    let mut connection_counter = 0;
    println!("server started ... v0.2.2");
    let _cloned_server_data_arctex: Arc<Mutex<ServerData>> = server_data_arctex.clone();

    while let (stream,_sock_addr) = server.accept().await.expect("accept 失败") {
        connection_counter += 1;
        let cloned_server_data_arctex: Arc<Mutex<ServerData>> = server_data_arctex.clone();
        tokio::spawn(async move{
            // 初始化锁 
            let server_data_arctex = cloned_server_data_arctex;
            let ws_thread_info_arctex = Arc::new(Mutex::new(WsThreadInfo::new(connection_counter)));
            println!("connected {}",ws_thread_info_arctex.lock().await.connect_id);
            let (write,read) = accept_async(stream).await.unwrap().split();
            let (write_arctex,read_arctex) = (Arc::new(Mutex::new(write)),Arc::new(Mutex::new(read)));

            let ctx = Context{
                ws_thread_info_arctex,
                server_data_arctex,
                write: write_arctex,
                read: read_arctex,
            };


            loop{
                let msg = read_sock(ctx.clone()).await;
                if msg.is_close(){ close_sock(ctx).await; break; }
                if msg.is_text() {
                    let msg = msg.into_text().unwrap();
                    println!("receive \"{}\"",msg);

                    let msg_args = MessageArgs::parse_message(msg);
                    match_command!(
                        command register_app with args (user_name:String,room_num:u32,board_id:usize) debug true
                            => after_app_confirmed
                        command register_board with args (wifi:String,ip:String) debug true
                            => after_board_confirmed
                        command register_referee with args (room_num:u32) debug true
                            => after_referee_confirmed
                        command register_visitor with args (room_num:u32) debug true
                            => after_visitor_confirmed
                        command log with args (log_str:String) debug false
                        command request_list_rooms with args () debug true
                        command request_list_boards with args () debug true
                        command request_get_model_type with args () debug true
                        in msg_args 
                        with context ctx
                    )
                }                
            }
        });
    }
}
async fn close_sock(ctx:Context){
    println!("connection {} closed",ctx.ws_thread_info_arctex.lock().await.connect_id);
    match ctx.ws_thread_info_arctex.lock().await.character {
        Character::Board { board_id} =>{
            let mut server_data = ctx.server_data_arctex.lock().await;
            debug_info_green!("remove board {} in room_num:{:?}",board_id, ctx.server_data_arctex.lock().await.boards.get(board_id).unwrap().op_room_num);
            server_data.boards.remove(board_id);
        },
        Character::App{   board_id }=> {
            debug_info_green!("app disconnected")
        },
        Character::NotSure =>{
            debug_info_green!("not sure disconnected")
        },
        Character::Referee { referee_id } => {
            debug_info_green!("referee disconnected")
        }
        Character::Visitor { visitor_id} => {
            debug_info_green!("visitor disconnected")
        },
    }
}
async fn read_sock(ctx:Context)->Message{
    // debug_info!("{}","try read ");
    let cloned_read_arctex = ctx.read.clone();
    // 半小时之内没有通讯就删了你
    let msg = match timeout(Duration::from_secs(1800),cloned_read_arctex.lock().await.next()).await{
        Ok(Some(Ok(m)))=>m,
        Err(_e)=>{
            println!("read 超时,断开连接");
            Message::Close(None)
        },
        _ => {
            Message::Close(None)
        }
    };
    msg
}
async fn after_app_confirmed(ctx:Context){
    loop{
        let msg = read_sock(ctx.clone()).await;
        if msg.is_close(){ close_sock(ctx.clone()).await; break; }
        if msg.is_text() {
            let msg = msg.into_text().unwrap();
            println!("receive \"{}\"",msg);
            let cmd_args = MessageArgs::parse_message(msg);
            match_command!(
                command request_list_rooms with args () debug true
                command request_list_boards with args () debug true
                command log with args (log_string:String)  debug false
                in cmd_args with context ctx
            )
        }
    }
}
async fn after_board_confirmed(ctx:Context){
    loop{
        // board 必须返回 check:() 才行
        let mut write = ctx.write.lock().await;
        let _ = write.send(Message::Text("request_check:()".to_string())).await;
        // debug_info!("send {}",Message::Text("request_check".to_string()) );
        let msg = match timeout(Duration::from_millis(3000),ctx.read.lock().await.next()).await{
            Ok(Some(Ok(m)))=>{
                m
            },
            Err(_e)=>{
                println!("read 超时,断开连接");
                Message::Close(None)
            },
            _ => {
                Message::Close(None)
            }
        };
        if let Message::Text(text)=msg{
            if text.contains("check:()"){
                // debug_info!("check")
                // should do nothing 
            }
        }else if let Message::Close(_) = msg{
            close_sock(ctx.clone()).await;
            break;
        }
        tokio::time::sleep(Duration::from_millis(3000)).await;
    }
}
async fn after_referee_confirmed(ctx:Context){
    loop{
        let msg = read_sock(ctx.clone()).await;
        if msg.is_close(){ close_sock(ctx.clone()).await; break; }
        if msg.is_text() {
            let msg = msg.into_text().unwrap();
            println!("receive \"{}\"",msg);
            let cmd_args = MessageArgs::parse_message(msg);
            match_command!(
                command request_list_rooms with args () debug true
                command request_list_boards with args () debug true
                command request_list_boards_in_room with args (room_num:u32) debug true
                command request_set_board_coords with args (board_id:usize,x:f32,y:f32) debug true
                command log with args (log_string:String)  debug false
                in cmd_args with context ctx
            )
        }
    }
}
async fn after_visitor_confirmed(ctx:Context){
    loop{
        let msg = read_sock(ctx.clone()).await;
        if msg.is_close(){ close_sock(ctx.clone()).await; break; }
        if msg.is_text() {
            let msg = msg.into_text().unwrap();
            println!("receive \"{}\"",msg);
            let cmd_args = MessageArgs::parse_message(msg);
            match_command!(
                // command request_list_rooms with args () debug true
                // command request_list_boards with args () debug true
                // command request_list_boards_in_room with args (room_num:u32) debug true
                // command request_set_board_coords with args (board_id:usize,x:f32,y:f32) debug true
                command log with args (log_string:String)  debug false
                in cmd_args with context ctx
            )
        }
    }
}