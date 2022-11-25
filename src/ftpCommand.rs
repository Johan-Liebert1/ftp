use std::io::{self, Read};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::str;

#[derive(Clone, Debug)]
pub enum FTPCommand {
    Auth,
    Syst,
    NoOp,
    Pwd,
    Type,
    Pasv,
    CdUp,
    List(PathBuf),
    Cwd(PathBuf),
    Mkd(PathBuf),
    Rmd(PathBuf),
    User(String),
    Unknown(String),
}

fn to_uppercase(data: &mut [u8]) {
    for byte in data {
        if *byte >= 'a' as u8 && *byte <= 'z' as u8 {
            *byte -= 32;
        }
    }
}

impl AsRef<str> for FTPCommand {
    fn as_ref(&self) -> &str {
        match &*self {
            FTPCommand::CdUp => "CDUP",
            FTPCommand::List(_) => "LIST",
            FTPCommand::Mkd(_) => "MKD",
            FTPCommand::Rmd(_) => "RMD",
            FTPCommand::Pasv => "PASV",
            FTPCommand::Type => "TYPE",
            FTPCommand::Auth => "AUTH",
            FTPCommand::Pwd => "PWD",
            FTPCommand::NoOp => "NOOP",
            FTPCommand::Cwd(_) => "CWD",
            FTPCommand::Syst => "SYST",
            FTPCommand::User(_) => "USER",
            FTPCommand::Unknown(_) => "UNKN",
        }
    }
}

impl FTPCommand {
    pub fn new(input: Vec<u8>) -> io::Result<Self> {
        let mut iter = input.split(|&byte| byte == b' ');
        let mut command = iter.next().expect("command in input").to_vec();

        to_uppercase(&mut command);

        let data = iter.next();

        let command = match command.as_slice() {
            b"RMD" => FTPCommand::Rmd(
                data.map(|bytes| Path::new(str::from_utf8(bytes).unwrap()).to_path_buf())
                    .unwrap(),
            ),

            b"MDK" => FTPCommand::Mkd(
                data.map(|bytes| Path::new(str::from_utf8(bytes).unwrap()).to_path_buf())
                    .unwrap(),
            ),

            b"CDUP" => FTPCommand::CdUp,

            b"TYPE" => FTPCommand::Type,

            b"PWD" => FTPCommand::Pwd,

            b"NOOP" => FTPCommand::NoOp,

            b"AUTH" => FTPCommand::Auth,

            b"SYST" => FTPCommand::Syst,

            b"PASV" => FTPCommand::Pasv,

            b"LIST" => FTPCommand::List(
                data.map(|bytes| Path::new(str::from_utf8(bytes).unwrap()).to_path_buf())
                    .unwrap(),
            ),

            b"USER" => FTPCommand::User(
                data.map(|bytes| {
                    String::from_utf8(bytes.to_vec()).expect("cannot convert bytes to String")
                })
                .unwrap_or_default(),
            ),

            b"CWD" => FTPCommand::Cwd(
                data.map(|bytes| Path::new(str::from_utf8(bytes).unwrap()).to_path_buf())
                    .unwrap(),
            ),

            s => FTPCommand::Unknown(str::from_utf8(s).unwrap_or("").to_owned()),
        };

        Ok(command)
    }
}

pub fn read_all_message(stream: &mut TcpStream) -> Vec<u8> {
    let buf = &mut [0; 1];
    let mut out = Vec::with_capacity(100);

    loop {
        match stream.read(buf) {
            Ok(received) if received > 0 => {
                if out.is_empty() && buf[0] == b' ' {
                    continue;
                }

                out.push(buf[0]);
            }

            _ => return Vec::new(),
        }

        let len = out.len();

        if len > 1 && out[len - 2] == b'\r' && out[len - 1] == b'\n' {
            out.pop();
            out.pop();
            return out;
        }
    }
}
