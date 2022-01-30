use std::io::{BufRead, Read, Write};

const SEND_BUFFER_SIZE: usize = 65536; // GET
const RECEIVE_BUFFER_SIZE: usize = 65536; // POST

pub fn listen<A: std::net::ToSocketAddrs>(address: A) -> std::io::Result<()> {
    let listener = std::net::TcpListener::bind(address)?;
    eprintln!("listening on {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        std::thread::spawn(move || {
            handle(stream)
        });
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

    loop {
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
        let mut keep_alive = false;
        for header in &headers {
            if header.starts_with("connection: keep-alive") { keep_alive = true; break; }
        }
        let keep_alive = keep_alive;

        eprintln!("{} {} {} keep_alive: {}", from_utf8(protocol)?, target, from_utf8(version)?, keep_alive);
        match protocol {
            b"OPTIONS" => {
                writer.write(b"HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: *\r\nAccess-Control-Allow-Methods: POST, GET\r\n\r\n")?;
                writer.flush()?;
            }
            b"GET" => {
                let length = parse::<u64>(target.chars().skip(1).collect::<String>())?;
                writer.write(b"HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/octet-stream\r\nContent-Length: ")?;
                writer.write(length.to_string().as_bytes())?;
                writer.write(b"\r\n\r\n")?;
                let mut write = 0;
                while write < length {
                    let _write = if (write + SEND_BUFFER_SIZE as u64) < length {
                        writer.write(&mut [0u8; SEND_BUFFER_SIZE])?
                    } else {
                        writer.write(&mut vec![0; (length - write) as usize])?
                    };
                    if _write == 0 { break; }
                    write = write + _write as u64;
                }
                writer.flush()?;
                eprintln!("wrote {}/{}", write, length);
            },
            b"POST" => {
                for header in &headers {
                    if !header.starts_with("content-length: ") { continue }

                    let length = parse(header.chars().skip(16).collect::<String>())?;
                    let mut read = 0;
                    while read < length {
                        let _read = if (read + RECEIVE_BUFFER_SIZE as u64) < length {
                            reader.read(&mut [0u8; RECEIVE_BUFFER_SIZE])?
                        } else {
                            reader.read(&mut vec![0; (length - read) as usize])?
                        };
                        if _read == 0 { break; }
                        read = read + _read as u64;
                    }
                    eprintln!("read {}/{}", read, length);
                    writer.write(b"HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\n\r\n")?;
                    writer.flush()?;
                    break
                }
            },
            _ => {
                writer.write(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n")?;
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("method not allowed: {} (must be GET or POST)", from_utf8(&protocol)?)));
            }
        };

        if !keep_alive { break; }
    }

    Ok(())
}

fn from_utf8(bytes: &[u8]) -> std::io::Result<&str> {
    match std::str::from_utf8(bytes) {
        Ok(str) => Ok(str),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid utf8: {}", e)))
    }
}