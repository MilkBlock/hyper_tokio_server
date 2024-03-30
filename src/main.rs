use board_info::BoardInfo;
use character::Character;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};

use player_app_info::PlayerAppInfo;
use referee_info::RefereeInfo;
use request::{SinkArctex, StreamArctex};
use room::Room;
use slab::Slab;
use tokio::join;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, timeout, Timeout};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};

mod request;
mod player_app_info;
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
use crate::request::{ check, log, register_app, register_board, register_referee, register_visitor, request_get_model_type, request_list_boards, request_list_boards_in_room, request_list_rooms, request_set_board_coords, Context};

const ONLINE_TIMEOUT_SECS:u64 = 4;
const REQUEST_ONLINE_CHECK_INTERVAL:u64 = 2;
pub struct ServerData{
    rooms : Vec<Room>,
    boards : Slab<BoardInfo>,
    visitors : Slab<VisitorInfo>,
    referees : Slab<RefereeInfo>,
    player_apps : Slab<PlayerAppInfo>,
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
    let mut _server_data = ServerData{rooms:Vec::from([Room::new(1)]),boards:Slab::new(),visitors:Slab::new(),referees:Slab::new(), player_apps: Slab::new()};
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
                let msg = read_sock_with_timeout(ctx.clone()).await;
                if msg.is_close(){ close_sock(ctx).await; break; }
                if msg.is_text() {
                    let msg = msg.into_text().unwrap();
                    println!("receive \"{}\"",msg);

                    let msg_args = MessageArgs::parse_message(msg);
                    match_command!(
                        command register_app with args (user_name:String,room_num:u32,board_id:usize) debug true
                            => after_app_confirmed,online_check
                        command register_board with args (wifi:String,ip:String) debug true
                            => after_board_confirmed,online_check
                        command register_referee with args (room_num:u32) debug true
                            => after_referee_confirmed,online_check
                        command register_visitor with args (room_num:u32) debug true
                            => after_visitor_confirmed,online_check
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
    let mut ws_thread_info = ctx.ws_thread_info_arctex.lock().await;
    println!("connection {} closed",ws_thread_info.connect_id);
    match ws_thread_info.character {
        Character::Board { board_id: board_idx} =>{
            let mut server_data = ctx.server_data_arctex.lock().await;
            debug_info_green!("remove board {} in room_num:{:?}",board_idx, server_data.boards.get(board_idx).expect(format!("找不到要删的 board {}",board_idx).as_str()).op_room_num);
            server_data.boards.remove(board_idx);
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
        Character::Exited => {debug_info_red!("你不能断开已经断开的连接char")},
    }
    ws_thread_info.character = Character::Exited;
}
async fn read_sock_with_timeout(ctx:Context)->Message{
    // debug_info!("{}","try read ");
    let cloned_read_arctex = ctx.read.clone();
    // 半小时之内没有通讯就删了你
    let msg = match timeout(Duration::from_secs(ONLINE_TIMEOUT_SECS),cloned_read_arctex.lock().await.next()).await{
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
async fn online_check(ctx:Context){
    debug_info_green!("online check started");
    loop {
        match ctx.write.lock().await.send(Message::text("request_check:()")).await{
            Ok(_) => {
                // do nothing 管杀不管埋
            },
            Err(_) => {}
            // debug_info_red!("无法发送 online check")
        }
        match ctx.ws_thread_info_arctex.lock().await.character{
            Character::Exited => {debug_info_green!("online_check exited" ); return;},
            _ => {},
        }
        sleep(Duration::from_secs(REQUEST_ONLINE_CHECK_INTERVAL)).await;
    }
}
async fn after_app_confirmed(ctx:Context){
    loop{
        // let msg = read_sock(ctx.clone()).await;
        let msg = match timeout(Duration::from_millis(2000),ctx.read.lock().await.next()).await{
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

        match msg{ 
            Message::Text(text) => {
                println!("receive \"{}\"",text);
                let cmd_args = MessageArgs::parse_message(text);
                match_command!(
                    command request_list_rooms with args () debug true
                    command request_list_boards with args () debug true
                    command log with args (log_string:String)  debug false
                    command check with args () debug true 
                    in cmd_args with context ctx
                )
            },
            Message::Close(_) => {
                close_sock(ctx.clone()).await; 
                break;
            },
            _=> panic!("无法识别此 message"), 
        }
    }
}
async fn after_board_confirmed(ctx:Context){
    loop{
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

        }else if let Message::Close(_) = msg{
            close_sock(ctx.clone()).await;
            break;
        }
    }
}
async fn after_referee_confirmed(ctx:Context){
    loop{
        let msg = read_sock_with_timeout(ctx.clone()).await;
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
        let msg = read_sock_with_timeout(ctx.clone()).await;
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