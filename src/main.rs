#![allow(unused_imports)]
use regex::Regex;
use std::borrow::Borrow;
use std::fs::{read, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
static FILENAME: &str = "foo.txt";

fn handle_input(mut stream: TcpStream, mut logger: File) -> std::io::Result<()> {
    let mut buffer = String::new();
    match stream.read_to_string(&mut buffer) {
        Ok(_) => {
            println!("Received: {:?}", buffer);
            // write to logger file
            let _ = logger.write_all(buffer.as_bytes())?;
        }
        Err(e) => eprintln!("Failed to read from client: {}", e),
    }
    Ok(())
}
fn handle_input_by_line(stream: TcpStream, mut logger: File) -> std::io::Result<()> {
    let reader = BufReader::new(stream.try_clone().unwrap());
    let lines = reader.lines();
    for line in lines {
        match line {
            Ok(buffer) => {
                println!("Received: {:?}", buffer);
                let _ = logger.write_all(&handle_buffer_fmt(buffer.as_bytes(), b"\r\n"));
            }
            Err(e) => eprintln!("Error while writing by line {}", e),
        }
    }
    Ok(())
}
fn create_logger_file() -> std::io::Result<()> {
    File::create(FILENAME)?;
    Ok(())
}

fn handle_buffer_fmt(fixed_array: &[u8], slice: &[u8]) -> Vec<u8> {
    let result: Vec<u8> = fixed_array
        .iter() // Create an iterator over the fixed array
        .chain(slice.iter()) // Chain with the slice's iterator
        .cloned() // Clone the values to get owned data
        .collect();
    return result;
}

fn main() -> std::io::Result<()> {
    // create tcp connection bound to localhost port 6379
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // create a logger file to insert inputs from connection
                let _ = create_logger_file();
                let logger = OpenOptions::new()
                    .append(true)
                    .open(FILENAME)
                    .expect("cannot open file");

                println!("accepted new connection {}", stream.peer_addr().unwrap());
                // make a new thread for every new connection and initiate logging
                thread::spawn(move || {
                    let _ = handle_input_by_line(stream, logger);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
