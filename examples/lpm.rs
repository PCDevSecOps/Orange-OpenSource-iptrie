use std::{env, io};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::io::Write;

use iptrie::*;

use std::net::{Ipv4Addr, Ipv6Addr};
use ipnet::{Ipv4Net,Ipv6Net};

// should be "end of line" (\n) terminated
static EMPTY_PREFIX: &str = "<empty prefix>\n";


fn main() {
    let mut handle = io::BufWriter::new(io::stdout());

    let filename = env::args().skip(1).next().expect("needs a LPM file");
    let file = File::open(filename).expect("can’t open LPM file");
    let length = file.metadata().unwrap().len() as usize;

    let lpmfile = unsafe {
        libc::mmap(null_mut(), length, libc::PROT_READ, libc::MAP_PRIVATE, file.as_raw_fd(), 0) as *const c_char
    };
    if lpmfile.is_null() {
        panic!("FATAL: can't load LPM file");
    }
    let lpmfile = unsafe { std::str::from_utf8(CStr::from_ptr(lpmfile).to_bytes()) }.unwrap();

    let mut map4 = RTrieMap::<Ipv4Net,&str>::with_root_and_capacity(EMPTY_PREFIX, 2000000);
    let mut map6 = RTrieMap::<Ipv6Net,&str>::with_root_and_capacity(EMPTY_PREFIX, 2000000);

    lpmfile.split_inclusive('\n').into_iter()
       // .take(100)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))// skip empty and comment lines
        .map(|s| (s, s.split_ascii_whitespace().into_iter().next().expect("bad formatted line")))
        .for_each(| (line, prefix) | {
            if let Ok(pfx) = prefix.parse::<Ipv4Net>() {
                map4.insert(pfx, line);
            } else if let Ok(pfx) = prefix.parse::<Ipv6Net>() {
                map6.insert(pfx, line);
            } else {
                eprintln!("WARN: skip bad formatted line: {}", line);
            }
        });

    //map4.open_dot_view();
    let map4 = map4.compress();
    let map6 = map6.compress();
   // map6.generate_graphviz_file("a".into());
    //map4.info();
   // map6.info();
    //std::process::exit(0);
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) =>  break,
            Ok(_) => {
                let input = &input[..(input.len()-1)];
                if let Ok(addr) = input.parse::<Ipv4Addr>() {
                    handle.write_all(map4.lookup(&addr).1.as_bytes()).unwrap();
                } else if let Ok(addr) = input.parse::<Ipv6Addr>() {
                    handle.write_all(map6.lookup(&addr).1.as_bytes()).unwrap();
                } else {
                    eprintln!("WARN: can’t parse '{}' (not an IP address)", input);
                }
            }
            Err(error) => panic!("{}", error)
        }
    }
}