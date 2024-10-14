#![allow(unused_imports)]
use bytes::buf;
use regex::Regex;
use std::borrow::Borrow;
use std::fs::{read, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
static FILENAME: &str = "foo.txt";

/* deprecated handle_input
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
*/

fn handle_buffer_resp(buffer: &[u8], stream: TcpStream) {
    let string_to_check = &String::from_utf8_lossy(buffer); //.trim().to_owned();

    // check if input is a command
    let cli_pattern = Regex::new(r"^(PING)|((ECHO|set|get|type|del)\s).*\r\n").unwrap();
    if cli_pattern.is_match(string_to_check.borrow()) {
        return handle_resp_commands(string_to_check, stream);
    }

    /* handle_resp_commands
        For Simple Strings, the first byte of the reply is "+"   +<string>\r\n
        For Errors, the first byte of the reply is "-"
        For Integers, the first byte of the reply is ":"
        For Bulk Strings, the first byte of the reply is "$"     $<length>\r\n<string>\r\n
        For Arrays, the first byte of the reply is "*"           *2\r\n$4\r\necho\r\n$11\r\nhello world\r\n
    */

    let string_pattern = Regex::new(r"^\+.+\r\n").unwrap();
    let error_pattern = Regex::new(r"^\-.+\r\n").unwrap();
    let integer_pattern = Regex::new(r"^\:\d+\r\n").unwrap();
    let bulk_string_pattern = Regex::new(r"^\$\d+\\r\\n.+\\r\\n").unwrap();
    let arrays_pattern = Regex::new(r"^\*\d+\r\n").unwrap();
    let null_pattern = Regex::new(r"^(\$-1\r\n|\*-1\r\n)").unwrap();

    if string_pattern.is_match(string_to_check) {
        println!("string_pattern match");
    } else if error_pattern.is_match(string_to_check) {
        println!("error_pattern match");
    } else if integer_pattern.is_match(string_to_check) {
        println!("integer_pattern match");
    } else if arrays_pattern.is_match(string_to_check) {
        println!("arrays_pattern match");
    } else if bulk_string_pattern.is_match(string_to_check) {
        println!("bulk_string_pattern match");
    } else if null_pattern.is_match(string_to_check) {
        println!("null_pattern match");
    } else {
        println!("input cant be matched")
    }
}

fn handle_resp_commands(string_to_check: &str, mut stream: TcpStream) {
    if Regex::new(r"^PING").unwrap().is_match(string_to_check) {
        let _ = stream.write_all(b"PONG\r\n");
    } else if Regex::new(r"^ECHO\s+.+").unwrap().is_match(string_to_check) {
        let _ = stream.write_all(string_to_check[4..].as_bytes());
    } else {
        let _ = stream.write_all(b"todo");
    }

    //  let echo_pattern =Regex::new(r"^ECHO\s.+\r\n").unwrap();
    return;
}

fn handle_input_by_line(stream: TcpStream, mut logger: File) -> std::io::Result<()> {
    let reader = BufReader::new(stream.try_clone().unwrap());
    let lines = reader.lines();
    for line in lines {
        match line {
            Ok(buffer) => {
                println!("Received: {:?}", buffer);
                // check if buffer matches with RESP protocol
                handle_buffer_resp(
                    &handle_buffer_fmt(buffer.as_bytes(), b"\r\n"),
                    stream.try_clone().unwrap(),
                );
                // log out to a file.
                let _ = logger.write_all(&handle_buffer_fmt(buffer.as_bytes(), b"\r\n"));
                // log out to client
                // let _ = stream.write_all(&handle_buffer_fmt(buffer.as_bytes(), b"\r\n"));
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
