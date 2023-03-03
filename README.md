<p align="center">
    <img height="300" alt="Redox" src="https://github.com/nthdeg/redox/blob/main/redox.png">
</p>

# REDOX #

Redox will be a post exploitation offensive toolset. As of now this is a reverse shell written in [Rust](https://www.rust-lang.org/) currently tested on Windows and Debian distros. It comes with the implant (client) which can be compiled for different platforms. Once you set your server ip in the client implant and run your server, the clients will connect one at a time and you will be able to interact with the connected devices one at a time.

You can read and write to disk, run executables and all the things you can normally do in a shell.

## Table of Contents

- [Redox](#redox)
  * [To Do](#todo)
  * [Tools in this repo](#tools-in-this-repo)
  * [Compiling the tools](#compiling-the-tools-in-this-repo)



## TODO:
- [X] Add File transfer
- [ ] Run implant in memory only
- [ ] Hide network traffic


## Tools in this repo

| File                                                                                                   | Description                                                                                                                                                                              |
|--------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [server](/server/src/main.rs)                                 | Server program that listens for client connections via TCP                                                                                                                                  |
| [client](/client/src/main.rs)                                 | Client program that connects back to server listening for connections via TCP                                                                                                                                  |

## Compiling the tools in this repo

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

For quickly running the project, go into each project directory and execute:  
```
`cargo run`
```

For compiling the project, go into each project directory and execute:  
```
`cargo build`
```
this will give you executables you can use portably in each environment: Windows and Linux.

If you want to build the final "release" version execute:  
```
`cargo build --release`
```
