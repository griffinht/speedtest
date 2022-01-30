use std::io::{BufRead, Read, Write};

static SEND_BUFFER_SIZE: i32 = 10000; // GET
static RECEIVE_BUFFER_SIZE: i32 = 10000; // POST

pub fn listen<A: std::net::ToSocketAddrs>(address: A) -> std::io::Result<()> {
    let listener = std::net::TcpListener::bind(address)?;
    eprintln!("listening on {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        match handle(stream) {
            Ok(_) => {}
            Err(e) => { eprintln!("{}", e); }
        }
    }
    Ok(())
}

fn read_until_exact<T: BufRead>(mut reader: T, byte: u8, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
    let read = reader.read_until(byte, buffer)?;
    if read < 1 {
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "not enough read"))
    } else {
        Ok(read)
    }
}

fn parse<T: std::str::FromStr>(string: String) -> std::io::Result<T> {
    match string.parse::<T>() {
        Ok(length) => Ok(length),
        Err(_) => { Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("error parsing {}", string)))}
    }
}
fn handle(stream: std::io::Result<std::net::TcpStream>) -> std::io::Result<()> {
    let stream = stream?;
    let address = stream.peer_addr().unwrap().ip();
    println!("{}", address);
    let mut reader = std::io::BufReader::new(stream.try_clone()?);
    let mut writer = std::io::BufWriter::new(stream.try_clone()?);

    let mut protocol = Vec::new();
    read_until_exact(&mut reader, b' ', &mut protocol)?;
    let protocol = &protocol[0..protocol.len() - 1];

    let mut target = Vec::new();
    read_until_exact(&mut reader, b' ', &mut target)?;
    let target = from_utf8(&target[0..target.len() - 1])?;

    let mut version = Vec::new();
    read_until_exact(&mut reader, b'\r', &mut version)?;
    read_until_exact(&mut reader, b'\n', &mut version)?;
    let version = &version[0..version.len() - 2];

    let mut headers = Vec::new();
    loop {
        let mut header = Vec::new();
        read_until_exact(&mut reader, b'\r', &mut header)?;
        read_until_exact(&mut reader, b'\n', &mut header)?;
        if header == b"\r\n" { break }
        let header = &header[0..header.len() - 2];
        headers.push(from_utf8(header)?.to_lowercase())
    }

    eprintln!("{} {} {}", from_utf8(protocol)?, target, from_utf8(version)?);
    match protocol {
        b"OPTIONS" => {
            writer.write(b"HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: *\r\nAccess-Control-Allow-Methods: POST, GET\r\n\r\n")?;
        }
        b"GET" => {
            let length = parse::<u64>(target.chars().skip(1).collect::<String>())?;
            writer.write(b"HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/octet-stream\r\nContent-Length: ")?;
            writer.write(length.to_string().as_bytes())?;
            writer.write(b"\r\n\r\n")?;
            let mut write = 0;
            while write < length {
                let mut buffer = [0u8; SEND_BUFFER_SIZE];
                let _write = writer.write(&mut buffer)?;
                if _write == 0 { break; }
                write = write + _write as u64;
            }
            eprintln!("wrote {}/{}", write, length);
        },
        b"POST" => {
            for header in headers {
                if !header.starts_with("content-length: ") { continue }

                let length = parse(header.chars().skip(16).collect::<String>())?;
                let mut read = 0;
                while read < length {
                    let mut buffer = [0u8; RECEIVE_BUFFER_SIZE];
                    let _read = reader.read(&mut buffer)?;
                    if _read == 0 { break; }
                    read = read + _read as u64;
                }
                eprintln!("read {}/{}", read, length);
                break
            }
            writer.write(b"HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\n\r\n")?;
        },
        _ => {
            writer.write(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n")?;
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("method not allowed: {} (must be GET or POST)", from_utf8(&protocol)?)));
        }
    };

    Ok(())
}

fn from_utf8(bytes: &[u8]) -> std::io::Result<&str> {
    match std::str::from_utf8(bytes) {
        Ok(str) => Ok(str),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid utf8: {}", e)))
    }
}