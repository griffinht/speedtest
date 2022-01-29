use std::io::{BufRead, Write};

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
        b"GET" => {
            writer.write(b"HTTP/1.1 200 OK\r\nContent-Length: ")?;
            writer.write(0.to_string().as_bytes())?;
            writer.write(b"\r\n\r\n")?;
        },
        b"POST" => {
            for header in headers {
                if !header.starts_with("content-length: ") { continue }

                let length: String = header.chars().skip(16).collect();
                eprintln!("{}", length);
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