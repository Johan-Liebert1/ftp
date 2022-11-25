use std::net::TcpListener;
use std::thread;

mod client;
mod fptResultCodes;
mod ftpCommand;
mod helpers;

#[macro_use]
extern crate cfg_if;

extern  crate time;

use client::handle_client;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1234").expect("Couldn't bind this address...");

    println!("Waiting for clients to connect...");

    for stream in &mut listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }

            Err(error) => {
                println!("A client tried to connect... {}", error);
            }
        }
    }
}
