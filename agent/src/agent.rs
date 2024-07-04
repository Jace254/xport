use crate::key_mouse;
use crate::screen::Cap;
use enigo::Enigo;
use enigo::{KeyboardControllable, MouseControllable };
use flate2::{
    Compression,
    write::DeflateEncoder
};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{
    channel,
    Sender, 
    Receiver
};
use std::time::Duration;
use rayon::prelude::*;
use std::thread;

pub fn run(host: String, pwd: String) {
    let mut hasher = DefaultHasher::new();
    hasher.write(pwd.as_bytes());
    let pk = hasher.finish();
    let suc = [
        (pk >> (7 * 8)) as u8,
        (pk >> (6 * 8)) as u8,
        (pk >> (5 * 8)) as u8,
        (pk >> (4 * 8)) as u8,
        (pk >> (3 * 8)) as u8,
        (pk >> (2 * 8)) as u8,
        (pk >> (1 * 8)) as u8,
        pk as u8,
    ];

    loop {
        let (tx4, rx) = channel::<TcpStream>();
        let tx_clone = tx4.clone();
        let host_clone = host.clone();

        thread::spawn(move || {
            connect_and_send(host_clone, tx_clone)
        });

        match handle_connection(rx, &suc) {
            Ok(()) => {
                println!("Break !");
                // Add a small delay before attempting to reconnect
                thread::sleep(Duration::from_secs(1));
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                // Add a small delay before attempting to reconnect
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

}

fn connect_and_send(host: String, tx: Sender<TcpStream>) {
    loop  {
        let hc = host.clone();

        // create stream channel
        match TcpStream::connect(hc) {
            Ok(mut stream) => {
                if stream.write_all(b"agent\n").is_ok() {
                    if tx.send(stream).is_ok() {
                        break;
                    }
                }
            }
            Err(_) => {
                thread::sleep(Duration::from_secs(1))
            }
        }

    }
}

fn handle_connection(rx: Receiver<TcpStream>, suc: &[u8; 8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = rx.recv()?;

    // Check connection validity
    let mut check = [0u8; 8];
    stream.read_exact(&mut check)?;
    println!("check {:?}", check);
    if suc != &check {
        println!("Password error");
        stream.write_all(&[2])?;
        return Ok(());
    }
    println!("Successfully accessed");
    stream.write_all(&[1])?;

    let ss = stream.try_clone()?;
    let th1 = thread::spawn(move || {
        if let Err(e) = std::panic::catch_unwind(|| {
            screen_stream(ss);
        }) {
            eprintln!("{:?}", e);
        }
    });

    let th2 = thread::spawn(move || {
        if let Err(e) = std::panic::catch_unwind(|| {
            event(stream);
        }) {
            eprintln!("{:?}", e);
        }
    });

    th1.join().unwrap();
    th2.join().unwrap();

    Ok(())
}

/**
 * Event handling
 */
fn event(mut stream: TcpStream) {
    let mut cmd = [0u8];
    let mut move_cmd = [0u8; 4];
    let mut enigo = Enigo::new();
    while let Ok(_) = stream.read_exact(&mut cmd) {
        println!("cmd: {:?}", cmd);
        match cmd[0] {
            common::KEY_UP => {
                stream.read_exact(&mut cmd).unwrap();
                if let Some(key) = key_mouse::key_to_enigo(cmd[0]) {
                    enigo.key_up(key);
                }
            }
            common::KEY_DOWN => {
                println!("Keydown");
                stream.read_exact(&mut cmd).unwrap();
                if let Some(key) = key_mouse::key_to_enigo(cmd[0]) {
                    enigo.key_down(key);
                }
            }
            common::MOUSE_KEY_UP => {
                stream.read_exact(&mut cmd).unwrap();
                if let Some(key) = key_mouse::mouse_to_enigo(cmd[0]) {
                    enigo.mouse_up(key);
                }
            }
            common::MOUSE_KEY_DOWN => {
                stream.read_exact(&mut cmd).unwrap();
                if let Some(key) = key_mouse::mouse_to_enigo(cmd[0]) {
                    enigo.mouse_down(key);
                }
            }
            common::MOUSE_WHEEL_UP => {
                enigo.mouse_scroll_y(-2);
            }
            common::MOUSE_WHEEL_DOWN => {
                enigo.mouse_scroll_y(2);
            }
            common::MOVE => {
                stream.read_exact(&mut move_cmd).unwrap();
                let x = ((move_cmd[0] as i32) << 8) | (move_cmd[1] as i32);
                let y = ((move_cmd[2] as i32) << 8) | (move_cmd[3] as i32);
                enigo.mouse_move_to(x, y);
            }
            _ => {
                return;
            }
        }
    }
}

/**
 * Encode data header
 */
#[inline]
fn encode(data_len: usize, res: &mut [u8]) {
    res[0] = (data_len >> 16) as u8;
    res[1] = (data_len >> 8) as u8;
    res[2] = data_len as u8;
    // res[3] = 10 as u8;
}

/*
Image byte order
+------------+
|     24     |
+------------+
|   length   |
+------------+
|   data     |
+------------+
length: data length
data: data
*/
fn screen_stream(mut stream: TcpStream) {
    let mut cap = Cap::new();

    let (w, h) = cap.wh();

    // Send w, h
    let mut meta = [0u8; 4];
    meta[0] = (w >> 8) as u8;
    meta[1] = w as u8;
    meta[2] = (h >> 8) as u8;
    meta[3] = h as u8;
    if let Err(_) = stream.write_all(&meta) {
        return;
    }
    let mut header = [0u8; 3];
    let mut yuv = Vec::<u8>::new();
    let mut last = Vec::<u8>::new();
    // First frame
    let bgra = cap.cap();
    common::convert::bgra_to_i420(w, h, bgra, &mut yuv);
    // Compress the delta frame
    let mut buf = Vec::<u8>::with_capacity(1024 * 4);
    let mut e = DeflateEncoder::new(buf, Compression::default());
    e.write_all(&yuv).unwrap();
    buf = e.reset(Vec::new()).unwrap();
    (last, yuv) = (yuv, last);
    // Send
    let clen = buf.len();
    encode(clen, &mut header);
    if let Err(_) = stream.write_all(&header) {
        return;
    }
    if let Err(_) = stream.write_all(&buf) {
        return;
    }
    loop {
        let bgra = cap.cap();
        unsafe {
            yuv.set_len(0);
        }
        common::convert::bgra_to_i420(w, h, bgra, &mut yuv);
        if yuv[..w*h] == last[..w*h] {
            continue;
        }
        // Use parallel processing for delta calculation
        last.par_iter_mut().zip(yuv.par_iter()).for_each(|(a, b)|{
            *a = *a ^ *b;
        });
        // Compress
        unsafe {
            buf.set_len(0);
        }
        e.write_all(&last).unwrap();
        buf = e.reset(buf).unwrap();
        (last, yuv) = (yuv, last);
        // Send
        let clen = buf.len();
        encode(clen, &mut header);
        if let Err(_) = stream.write_all(&header) {
            return;
        }
        if let Err(_) = stream.write_all(&buf) {
            return;
        }
    }
}
