use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use futures::future::FutureExt;
use slab::Slab;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, timeout};
use tokio_tungstenite::tungstenite::{accept, connect, Message, WebSocket};
use tokio_tungstenite::{accept_async, WebSocketStream};

mod request;
mod command_args;
mod macros;
use tokio::sync::{Mutex};
use std::sync::{Arc};
use std::fmt::Debug;
use std::time::Duration;

use crate::command_args::CommandArgs;
use crate::request::{ log, register_app, register_board, request_list_boards, request_list_rooms};
pub type Sink = SplitSink<WebSocketStream<TcpStream>,Message>;
pub type Stream = SplitStream<WebSocketStream<TcpStream>>;
pub type SinkArctex = Arc<Mutex<Sink>>;
pub type StreamArctex = Arc<Mutex<Stream>>;
pub struct ServerData{
    rooms : Vec<i32>,
    boards : Slab<BoardInfo>,
}
pub struct BoardInfo{
    wifi:String,
    ip:String,
    write: SinkArctex,
    read: StreamArctex,
}
impl Debug for BoardInfo{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoardInfo").field("wifi", &self.wifi).field("ip", &self.ip).finish()
    }
}
#[derive(Debug)]
enum BoardOrApp{
    // 这个在 register board 中指定
    Board{
        board_id : usize,
    },
    App,
    NotSure,
}
pub struct WsThreadInfo{
    board_or_app :BoardOrApp,
    connect_id : u32,
}
impl WsThreadInfo{
    pub fn new(connect_id:u32) -> Self{
        WsThreadInfo{
            board_or_app: BoardOrApp::NotSure,
            connect_id
        }
    }
}


/// A WebSocket server
#[tokio::main]
async fn main () {
    let server = TcpListener::bind("0.0.0.0:11451").await.expect("bind 端口失败");
    // ? 服务端端口注册完毕，进行数据初始化
    let mut _server_data = ServerData{rooms:Vec::from([1,2,3]),boards:Slab::new()};
    let server_data_arctex =Arc::new(Mutex::new(_server_data));
    let mut connection_counter = 0;
    println!("server started ... v0.2.2");
    let cloned_server_data_arctex: Arc<Mutex<ServerData>> = server_data_arctex.clone();

    while let (stream,sock_addr) = server.accept().await.expect("accept 失败") {
        connection_counter += 1;
        let cloned_server_data_arctex: Arc<Mutex<ServerData>> = server_data_arctex.clone();
        tokio::spawn(async move{
            // 初始化锁 
            let server_data_arctex = cloned_server_data_arctex;
            let ws_thread_info_arctex = Arc::new(Mutex::new(WsThreadInfo::new(connection_counter)));
            println!("connected {}",ws_thread_info_arctex.lock().await.connect_id);
            let (write,read) = accept_async(stream).await.unwrap().split();
            let (write_arctex,read_arctex) = (Arc::new(Mutex::new(write)),Arc::new(Mutex::new(read)));

            loop{
                let msg = read_sock(ws_thread_info_arctex.clone(), server_data_arctex.clone(), read_arctex.clone()).await;
                if msg.is_close(){ close_sock(ws_thread_info_arctex, server_data_arctex).await; break; }
                if msg.is_text() {
                    let msg = msg.into_text().unwrap();
                    println!("receive \"{}\"",msg);

                    let cmd_args = CommandArgs::parse_command(msg);
                    match_command!(
                        command register_app with args (user_name:String,room_num:i32)
                            => after_app_confirmed
                        command register_board with args (wifi:String,ip:String)
                            => after_board_confirmed
                        command log with args (log_str:String)
                        command request_list_rooms with args ()
                        command request_list_boards with args ()
                        in cmd_args with context (write_arctex,read_arctex,server_data_arctex,ws_thread_info_arctex)
                    )
                }                
            }
        });
    }
}
async fn close_sock(ws_thread_info_arctex : Arc<Mutex<WsThreadInfo>>, server_data_arctex :Arc<Mutex<ServerData>>){
    println!("connection {} closed",ws_thread_info_arctex.lock().await.connect_id);
    match ws_thread_info_arctex.lock().await.board_or_app {
        BoardOrApp::Board { board_id } =>{
            let mut server_data = server_data_arctex.lock().await;
            server_data.boards.remove(board_id);
            debug_info!("remove board {}",board_id)
        },
        BoardOrApp::App => {
            debug_info!("connection is app so do nothing")
        },
        BoardOrApp::NotSure =>{
            debug_info!("connection is not sure so do nothing")}
    }
}
async fn read_sock(ws_thread_info : Arc<Mutex<WsThreadInfo>>, server_data_arctex :Arc<Mutex<ServerData>>,read_arctex:StreamArctex)->Message{
    // debug_info!("{}","try read ");
    let cloned_read_arctex = read_arctex.clone();
    // 两分钟之内没有通讯就删了你
    let msg = match timeout(Duration::from_secs(1800),cloned_read_arctex.lock().await.next()).await{
        Ok(Some(Ok(m)))=>m,
        Err(e)=>{
            println!("read 超时,断开连接");
            Message::Close(None)
        },
        _ => {
            Message::Close(None)
        }
    };
    msg
}
async fn after_app_confirmed(ws_thread_info_arctex : Arc<Mutex<WsThreadInfo>>, server_data_arctex :Arc<Mutex<ServerData>>,write_arctex:SinkArctex,read_arctex:StreamArctex){
    loop{
        let msg = read_sock(ws_thread_info_arctex.clone(), server_data_arctex.clone(), read_arctex.clone()).await;
        if msg.is_close(){ close_sock(ws_thread_info_arctex.clone(), server_data_arctex).await; break; }
        if msg.is_text() {
            let msg = msg.into_text().unwrap();
            println!("receive \"{}\"",msg);
            let cmd_args = CommandArgs::parse_command(msg);
            match_command!(
                command request_list_rooms with args () 
                command request_list_boards with args () 
                command log with args (log_string:String) 
                in cmd_args with context (write_arctex,read_arctex,server_data_arctex,ws_thread_info_arctex)
            )
        }
    }
}
async fn after_board_confirmed(ws_thread_info_arctex : Arc<Mutex<WsThreadInfo>>, server_data_arctex :Arc<Mutex<ServerData>>,write_arctex:SinkArctex,read_arctex:StreamArctex){
    loop{
        // board 必须返回 check:() 才行
        let mut write = write_arctex.lock().await;
        write.send(Message::Text("request_check:()".to_string())).await;
        // debug_info!("send {}",Message::Text("request_check".to_string()) );
        let msg = match timeout(Duration::from_millis(3000),read_arctex.lock().await.next()).await{
            Ok(Some(Ok(m)))=>{
                m
            },
            Err(e)=>{
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
            close_sock(ws_thread_info_arctex, server_data_arctex).await;
            break;
        }
        tokio::time::sleep(Duration::from_millis(3000)).await;
    }
}