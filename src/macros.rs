#[macro_export] 
macro_rules! match_command{
    ($(command $command_name:ident  with args ($($arg_n:ident:$arg_n_type:ident),*) $(=> $next_phase:ident)? )+  in $command_args:ident with context ($write:ident,$read:ident,$data_arctex:ident,$ws_thread_info:ident)) => {
        match($command_args){
            $(Some(CommandArgs{command ,args}) if &command == stringify!($command_name) =>{
                let mut count = 0;
                match ($({count+=1;&args[count-1].parse::<$arg_n_type>()}),*){
                    ($(Ok($arg_n)),*) =>{
                        $command_name($ws_thread_info.clone(),$data_arctex.clone(),$write.clone(),$read.clone(), $($arg_n),*).await;
                        println!("successfullly run \x1B[32m {} \x1B[0m with args \x1B[32m{:?}\x1B[0m",command,args);
                        $(tokio::spawn($next_phase($ws_thread_info.clone(),$data_arctex.clone(),$write.clone(),$read.clone())).await;break;)?
                    },
                    _ => {println!("conversion failed \x1B[31m{}\x1B[0m with args \x1B[31m{:?}\x1B[0m ",stringify!($command_name), args)}
                }
            }),*
            // ? 找不到命令
            _=>{
                println!("command {:?} not recognized! " , $command_args)
            }
        }
    } ;
}
#[macro_export ]
macro_rules! debug_info{
    ($($t:tt)*) => {{
        println!("\x1B[34m debuginfo {}\x1B[0m",format!($($t)*))
    }};
}
// macro_rules! std_read_loop {
//     (before before_read:block read read:block parse parse:block process process:blcok) => {
//         {$before}
//         let msg = {$read}
//         let cmd_args = CommandArgs::parse_command(msg);

//     };
// }

// Some(CommandArgs{command ,args}) if command == "register" =>{
//     match (&args[0].parse::<String>(),&args[1].parse::<i32>()){
//         (Ok(user_name),Ok(room_num)) =>{
//             register(&mut ws,&cloned_server_data_arc_mtx, user_name, room_num)
//         }
//         _ => {println!("conversion failed with {:?} ",args)}
//     }
// },
