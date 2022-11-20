use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

mod fptResultCodes;
mod ftpCommand;

use fptResultCodes::ResultCode;

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

fn handle_client(stream: &mut TcpStream) {
    println!("New client!");
    if let Err(_) = stream.write(b"hello") {
        println!("Failed to send hello... :'(");
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1234").expect("Couldn't bind this address...");

    println!("Waiting for clients to connect...");

    for stream in &mut listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    handle_client(&mut stream);
                });
            }

            Err(error) => {
                println!("A client tried to connect... {}", error);
            }
        }
    }
}
