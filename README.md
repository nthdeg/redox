<p align="center">
    <img height="300" alt="Redox" src="https://github.com/nthdeg/redox/blob/main/redox.png">
</p>

# REDOX #

Redox is a C2 framework in its infancy written using [Rust](https://www.rust-lang.org/) and has been tested on Windows and Debian distros. It comes with the implant (client) which can be compiled for different platforms. Once you set your server ip in the client implant and run your server, the clients will connect one at a time and you will be able to interact with the connected devices one at a time.

When the implant is ran as user level, you can read and write to disk, run executables, download & transfer files from the C2 or remote locations and all the things you can normally do in a shell.


## Table of Contents

- [Redox](#redox)
  * [To Do](#todo)
  * [Features](#features)
  * [Tools in this repo](#tools-in-this-repo)
  * [Compiling the tools](#compiling-the-tools)
  * [Using the tools](#using-the-tools)



## TODO:
- [X] Add File transfer
- [ ] Organize and display all current connections
- [ ] Run implant in memory only
- [ ] Hide network traffic
- [ ] Add tty support, for now tee relevant commands

## Features:
- Upload and download files to/from agents
- Build native OS agents from Server (Windows and Linux)
- Local Staging Shell
- Keylogger (Windows agents)

## Programs in this repo

| File                                                                                                   | Description                                                                                                                                                                              |
|--------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [server](/server/src/main.rs)                                 | Server program that listens for client connections via TCP                                                                                                                                  |
| [client](/client/src/main.rs)                                 | Client program that connects back to server listening for connections via TCP                                                                                                                                  |

## Compiling the tools

This repository does not provide binaries, you will need to compile them yourself.  

[Install Rust](https://www.rust-lang.org/tools/install)  
Follow instructions for your platform and install. Make sure environment variables are correct and that you have a config.toml file in your cargo directory if needed.

## Linux Users
Linux users make sure you have pkg-config installed
```
sudo apt install pkg-config
```

This is the basic structure of all project folders:

```bash  
project
├── Cargo.toml
└── src
    └── main.rs
```

Cargo.toml contains the dependencies and the configuration for the compilation.
main.rs is the main file that will be compiled along with any directories that contain libraries.

For compiling the project, go into each project directory and execute:  
```
cargo build
```
this will give you executables you can use portably in each environment: Windows and Linux.

If you want to build the final "release" version execute:  
```
cargo build --release
```
## Using the Tools

After running the C2 server, type `build` at the home menu to set ip and port for the agent to connect back to (C2 server public IP, port is defaulted to 9001), the agent program (implant) is available in the client/target/release folder. Note this will build native client of OS in operation. Will be adding cross compiling soon.

Type `rtfm` to get list of available commands.

Place implant on desired device and run the program.

For quickly running the project, go into each project directory and execute:  
```
cargo run
```
