use std::io::{self, Read};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::str;

#[derive(Clone, Debug)]
enum Command {
    Auth,
    Cwd(PathBuf),
    Unknown(String),
}

fn to_uppercase(data: &mut [u8]) {
    for byte in data {
        if *byte >= 'a' as u8 && *byte <= 'z' as u8 {
            *byte -= 32;
        }
    }
}

impl AsRef<str> for Command {
    fn as_ref(&self) -> &str {
        match &*self {
            Command::Auth => "AUTH",
            Command::Cwd(path) => "CWD",
            Command::Unknown(_) => "UNKN",
        }
    }
}

impl Command {
    pub fn new(input: Vec<u8>) -> io::Result<Self> {
        let mut iter = input.split(|&byte| byte == b' ');
        let mut command = iter.next().expect("command in input").to_vec();

        to_uppercase(&mut command);

        let data = iter.next();

        let command = match command.as_slice() {
            b"AUTH" => Command::Auth,

            b"CWD" => Command::Cwd(
                data.map(|bytes| Path::new(str::from_utf8(bytes).unwrap()).to_path_buf())
                    .unwrap(),
            ),

            s => Command::Unknown(str::from_utf8(s).unwrap_or("").to_owned()),
        };

        Ok(command)
    }
}

fn read_all_message(stream: &mut TcpStream) -> Vec<u8> {
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
