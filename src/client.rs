use std::io::Write;
use std::{net::TcpStream, path::PathBuf};

use crate::fptResultCodes::ResultCode;
use crate::ftpCommand::read_all_message;
use crate::ftpCommand::FTPCommand;

#[allow(dead_code)]
struct Client {
    cwd: PathBuf,
    stream: TcpStream,
    name: Option<String>,
}

impl Client {
    fn new(stream: TcpStream) -> Client {
        Client {
            cwd: PathBuf::from("/"),
            stream,
            name: None,
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

            _ => {
                unimplemented!()
            }
        }
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
