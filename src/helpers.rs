use std::{collections::HashMap, fs::Metadata, io::Write, net::TcpStream, path::PathBuf};

cfg_if! {
    if #[cfg(windows)] {
        fn get_file_info(meta: &Metadata) -> u64 {
            use std::os::windows::prelude::*;
            (time::at(time::Timespec::new(meta.last_write_time())), meta.file_size())
        }
    } else {
        fn get_file_info(meta: &Metadata) -> (time::Tm, u64) {
            use std::os::unix::prelude::*;
            (time::at(time::Timespec::new(meta.mtime(), 0)), meta.size())
        }
    }
}

pub fn send_data(stream: &mut TcpStream, s: &str) {
    write!(stream, "{}", s).unwrap();
}

pub fn add_file_info(path: PathBuf, out: &mut String) {
    let months: HashMap<usize, &str> = HashMap::from([
        (1, "January"),
        (2, "February"),
        (3, "March"),
        (4, "April"),
        (5, "May"),
        (6, "June"),
        (7, "July"),
        (8, "August"),
        (9, "September"),
        (10, "October"),
        (11, "November"),
        (12, "December"),
    ]);

    let extra = if path.is_dir() { "/" } else { "" };
    let is_dir = if path.is_dir() { "d" } else { "-" };
    let meta = match ::std::fs::metadata(&path) {
        Ok(meta) => meta,
        _ => return,
    };
    let (time, file_size) = get_file_info(&meta);
    let path = match path.to_str() {
        Some(path) => match path.split("/").last() {
            Some(path) => path,
            _ => return,
        },
        _ => return,
    };
    let rights = if meta.permissions().readonly() {
        "r--r--r--"
    } else {
        "rw-rw-rw-"
    };
    let file_str = format!(
        "{is_dir}{rights} {links} {owner} {group} {size} {month} {day} {hour}:{min} {path}{extra}\r\n",
        is_dir = is_dir,
        rights = rights,
        links = 1,           // number of links
        owner = "anonymous", // owner name
        group = "anonymous", // group name
        size = file_size,
        month = months[&(time.tm_mon as usize)],
        day = time.tm_mday,
        hour = time.tm_hour,
        min = time.tm_min,
        path = path,
        extra = extra
    );
    out.push_str(&file_str);
    println!("==> {:?}", &file_str);
}
