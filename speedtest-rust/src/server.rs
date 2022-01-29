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

fn handle(stream: std::io::Result<std::net::TcpStream>) -> std::io::Result<()> {
    let stream = stream?;
    let address = stream.peer_addr().unwrap().ip();
    println!("{}", address);
    let mut reader = std::io::BufReader::new(stream.try_clone()?);
    let mut writer = std::io::BufWriter::new(stream.try_clone()?);

    let mut protocol = Vec::new();
    reader.read_until(b' ', &mut protocol)?;
    let protocol = &protocol[0..protocol.len() - 1];

    let mut target = Vec::new();
    reader.read_until(b' ', &mut target)?;
    let target = from_utf8(&target[0..target.len() - 1])?;

    let mut version = Vec::new();
    reader.read_until(b'\r', &mut version)?;
    reader.read_until(b'\n', &mut version)?;
    let version = &version[0..version.len() - 1];

    eprintln!("{} {} {}", from_utf8(protocol)?, target, from_utf8(version)?);
    match protocol {
        b"GET" => {
            writer.write(b"HTTP/1.1 200 OK\r\nContent-Length: ")?;
            writer.write(0.to_string().as_bytes())?;
            writer.write(b"\r\n\r\n")?;
        },
        b"POST" => {
            writer.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
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