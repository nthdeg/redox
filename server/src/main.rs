use std::process::{Command,exit};
use std::io::{self, Write, BufReader, BufRead};
use std::net::Ipv4Addr;

// server program with multi connections
// read write privs anywhere on disk if ran as current user

    use std::mem::drop;
    use std::net::{Shutdown, SocketAddrV4, TcpListener, TcpStream};

    fn handle_connection(clientsocket: &mut TcpStream){
        println!("Client connected: {}", clientsocket.local_addr().unwrap());
        let clientaddr = clientsocket.peer_addr().unwrap();
        println!(" 
        _______________________________________________________

        ███████████   ██████████ ██████████      ███████    █████ █████
        ░░███░░░░░███ ░░███░░░░░█░░███░░░░███   ███░░░░░███ ░░███ ░░███ 
         ░███    ░███  ░███  █ ░  ░███   ░░███ ███     ░░███ ░░███ ███  
         ░██████████   ░██████    ░███    ░███░███      ░███  ░░█████   
         ░███░░░░░███  ░███░░█    ░███    ░███░███      ░███   ███░███  
         ░███    ░███  ░███ ░   █ ░███    ███ ░░███     ███   ███ ░░███ 
         █████   █████ ██████████ ██████████   ░░░███████░   █████ █████
        ░░░░░   ░░░░░ ░░░░░░░░░░ ░░░░░░░░░░      ░░░░░░░    ░░░░░ ░░░░░ 
                                                                        
         _______________________________________________________       

         Type 'rtfm' for a menu of built in shortcuts

         If multiple clients are connected, you can exit current sessions by typing 'quit' to switch to the next connected client        

          Client connected: {} <- {}\n", clientsocket.local_addr().unwrap(),clientaddr);
    
    loop {
        println!("Enter Command to send: ");
        let mut msg = String::new();
        io::stdin().read_line(&mut msg).expect("String expected");
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
        } else {
            msg.push('\0');
            let mut buffer: Vec<u8> = Vec::new();
            clientsocket.write(msg.as_bytes());
            println!("Sent {}", &msg);
            let mut reader = BufReader::new(clientsocket.try_clone().unwrap());
        }
        if msg.trim().contains("rtfm"){ 
            println!("THE MANUAL_________________________________________________________________\n");
            if cfg!(windows) {
                println!("Usage: [COMMAND]           Gives result\n");
                println!(" dl,                       Asks for source url and filename to write\n");
                println!(" quit,                     Quits current client connection\n");
            } else if cfg!(unix) { 
                println!("Usage: man [OPTION...] [SECTION] PAGE...\n");
                println!(" dl,                       Asks for source url and filename to write\n");
                println!(" quit,                     Displays quits current client connection\n");
            } else if cfg!(target_os = "macos") {
                println!("Usage: man [OPTION...] [SECTION] PAGE...\n");
                println!(" dl,                       Asks for source url and filename to write\n");
                println!(" quit,                     Displays quits current client connection\n");
            }
        }
        msg.push('\0');
        let mut buffer: Vec<u8> = Vec::new();
        if msg.trim().contains("quit"){
            println!("shutting down client stream: {}", &clientaddr);
            clientsocket.shutdown(Shutdown::Both);
            println!("end of connections, crtl + c to terminate server program: {}", clientsocket.local_addr().unwrap());
            break;
        }
        // shortcut for help in all platforms 
        
        let mut reader = BufReader::new(clientsocket.try_clone().unwrap());
        reader.read_until(b'\0', &mut buffer);
        println!("client {} sent \n{}", clientaddr, String::from_utf8_lossy(&buffer));
    }
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

use std::thread;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    
    let ip = "0.0.0.0".parse::<Ipv4Addr>().unwrap();
    let mut s = SocketAddrV4::new(ip, 5358);

    println!("IP Address: {}", s.ip());
    println!("Port: {}", s.port());

    let listener = TcpListener::bind(s);

    let listener: () = match listener {
            Ok(l) => {
                println!("Successfully binded to {}", l.local_addr().unwrap());
            }
            Err(e) => {println!("{}",e); exit(0);}
    };
 
    let listener = TcpListener::bind(format!("{}:5358", ip)).unwrap();

    // v2 using multi threading
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move|| {
                    // connection succeeded
                    handle_connection(&mut stream);
                });
            }
            Err(e) => { /* connection failed */ }
        }
    } 
    println!("Stopping server listener");
    drop(listener);
}
