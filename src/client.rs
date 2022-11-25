use std::env;
use std::fs::{create_dir, read_dir, remove_dir_all};
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::path::Path;
use std::{net::TcpStream, path::PathBuf};

use crate::fptResultCodes::ResultCode;
use crate::ftpCommand::read_all_message;
use crate::ftpCommand::FTPCommand;
use crate::helpers::{add_file_info, send_data};

#[allow(dead_code)]
struct Client {
    cwd: PathBuf,
    stream: TcpStream,
    name: Option<String>,
    data_writer: Option<TcpStream>,
}

impl Client {
    fn new(stream: TcpStream) -> Client {
        Client {
            cwd: PathBuf::from("/"),
            stream,
            name: None,
            data_writer: None,
        }
    }

    fn handle_cmd(&mut self, cmd: FTPCommand) {
        println!("====> {:?}", cmd);

        match cmd {
            FTPCommand::Auth => send_cmd(
                &mut self.stream,
                ResultCode::CommandNotImplemented,
                "Not implemented",
            ),

            FTPCommand::Syst => send_cmd(&mut self.stream, ResultCode::Ok, "I won't tell"),

            FTPCommand::User(username) => {
                if username.is_empty() {
                    send_cmd(
                        &mut self.stream,
                        ResultCode::InvalidParameterOrArgument,
                        "Invalid username",
                    )
                } else {
                    self.name = Some(username.to_owned());
                    send_cmd(
                        &mut self.stream,
                        ResultCode::UserLoggedIn,
                        &format!("Welcome {}!", username),
                    );
                }
            }

            FTPCommand::Unknown(_) => send_cmd(
                &mut self.stream,
                ResultCode::UnknownCommand,
                "Not implemented",
            ),

            FTPCommand::NoOp => send_cmd(&mut self.stream, ResultCode::Ok, "Doing nothing..."),

            FTPCommand::Type => send_cmd(
                &mut self.stream,
                ResultCode::Ok,
                "Transfer type changed successfully",
            ),

            FTPCommand::Pwd => {
                let msg = format!("{}", self.cwd.to_str().unwrap_or(""));

                if !msg.is_empty() {
                    send_cmd(
                        &mut self.stream,
                        ResultCode::PATHNAMECreated,
                        &format!("\"/{}\" ", msg),
                    )
                } else {
                    send_cmd(
                        &mut self.stream,
                        ResultCode::FileNotFound,
                        "No such file or directory",
                    )
                }
            }

            FTPCommand::Pasv => {
                if self.data_writer.is_some() {
                    send_cmd(
                        &mut self.stream,
                        ResultCode::DataConnectionAlreadyOpen,
                        "Already listening...",
                    )
                } else {
                    let port = 43210;

                    send_cmd(
                        &mut self.stream,
                        ResultCode::EnteringPassiveMode,
                        &format!("127,0,0,1,{},{}", port >> 8, port & 0xFF),
                    );

                    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

                    let listener = TcpListener::bind(&addr).unwrap();

                    match listener.incoming().next() {
                        Some(Ok(client)) => {
                            self.data_writer = Some(client);
                        }

                        _ => {
                            send_cmd(
                                &mut self.stream,
                                ResultCode::ServiceNotAvailable,
                                "issues happen...",
                            );
                        }
                    }
                }
            }

            FTPCommand::List(path) => {
                // To get rid of borrow checker's nagging
                if self.data_writer.is_none() {
                    send_cmd(
                        &mut self.stream,
                        ResultCode::ConnectionClosed,
                        "No opened data connection",
                    );

                    return;
                }

                let server_root = env::current_dir().unwrap();
                let path = self.cwd.join(path);
                let directory = PathBuf::from(&path);

                if let Ok(path) = self.complete_path(directory, &server_root) {
                    send_cmd(
                        &mut self.stream,
                        ResultCode::DataConnectionAlreadyOpen,
                        "Starting to list directory...",
                    );

                    let mut out = String::new();

                    if path.is_dir() {
                        for entry in read_dir(path).unwrap() {
                            // for entry in dir {
                            if let Ok(entry) = entry {
                                add_file_info(entry.path(), &mut out);
                                // }
                            }

                            send_data(self.data_writer.as_mut().unwrap(), &out)
                        }
                    } else {
                        add_file_info(path, &mut out)
                    }
                } else {
                    send_cmd(
                        &mut self.stream,
                        ResultCode::InvalidParameterOrArgument,
                        "No such file or directory...",
                    );
                }

                if self.data_writer.is_some() {
                    self.data_writer = None;

                    send_cmd(
                        &mut self.stream,
                        ResultCode::ClosingDataConnection,
                        "Transfer done",
                    );
                }
            }

            FTPCommand::Cwd(path) => {
                self.cwd(path);
            }

            FTPCommand::CdUp => {
                if let Some(path) = self.cwd.parent().map(Path::to_path_buf) {
                    self.cwd = path;
                }
                send_cmd(&mut self.stream, ResultCode::Ok, "Done");
            }

            FTPCommand::Mkd(path) => self.mkd(path),
            FTPCommand::Rmd(path) => self.rmd(path),
        }
    }

    fn rmd(&mut self, path: PathBuf) {
        let server_root = env::current_dir().unwrap();

        if let Ok(path) = self.complete_path(path, &server_root) {
            if remove_dir_all(path).is_ok() {
                send_cmd(
                    &mut self.stream,
                    ResultCode::RequestedFileActionOkay,
                    "Folder successfully removed!",
                );

                return;
            }
        }

        send_cmd(
            &mut self.stream,
            ResultCode::FileNotFound,
            "Couldn't remove folder!",
        );
    }

    fn mkd(&mut self, path: PathBuf) {
        let server_root = env::current_dir().unwrap();
        let path = self.cwd.join(&path);

        if let Some(parent) = path.parent().map(|p| p.to_path_buf()) {
            if let Ok(mut dir) = self.complete_path(parent, &server_root) {
                if dir.is_dir() {
                    if let Some(filename) = path.file_name().map(|p| p.to_os_string()) {
                        dir.push(filename);

                        if create_dir(dir).is_ok() {
                            send_cmd(
                                &mut self.stream,
                                ResultCode::PATHNAMECreated,
                                "Folder successfully created!",
                            );
                            return;
                        }
                    }
                }
            }
        }

        send_cmd(
            &mut self.stream,
            ResultCode::FileNotFound,
            "Couldn't create folder",
        );
    }

    fn complete_path(&self, path: PathBuf, server_root: &PathBuf) -> Result<PathBuf, io::Error> {
        let directory = server_root.join(if path.has_root() {
            path.iter().skip(1).collect()
        } else {
            path
        });

        let dir = directory.canonicalize();

        if let Ok(ref dir) = dir {
            if !dir.starts_with(&server_root) {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
        }
        dir
    }

    fn cwd(&mut self, directory: PathBuf) {
        let server_root = env::current_dir().unwrap();
        let path = self.cwd.join(&directory);

        if let Ok(dir) = self.complete_path(path, &server_root) {
            if let Ok(prefix) = dir.strip_prefix(&server_root).map(|p| p.to_path_buf()) {
                self.cwd = prefix.to_path_buf();

                send_cmd(
                    &mut self.stream,
                    ResultCode::Ok,
                    &format!("Directory changed to \"{}\"", directory.display()),
                );

                return;
            }
        }

        send_cmd(
            &mut self.stream,
            ResultCode::FileNotFound,
            "No such file or directory",
        );
    }
}

fn send_cmd(stream: &mut TcpStream, code: ResultCode, message: &str) {
    let msg = if message.is_empty() {
        let code = ResultCode::CommandNotImplemented;
        format!("{}\r\n", code as u32)
    } else {
        format!("{} {}\r\n", code as u32, message)
    };

    println!("<==== {}", msg);

    write!(stream, "{}", msg).unwrap()
}

pub fn handle_client(mut stream: TcpStream) {
    println!("new client connected!");

    send_cmd(
        &mut stream,
        ResultCode::ServiceReadyForNewUser,
        "Welcome to this FTP server!",
    );

    let mut client = Client::new(stream);

    loop {
        let data = read_all_message(&mut client.stream);

        if data.is_empty() {
            println!("client disconnected...");
            break;
        }

        client.handle_cmd(FTPCommand::new(data).unwrap());
    }
}
