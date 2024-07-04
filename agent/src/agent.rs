use crate::key_mouse;
use crate::screen::Cap;
use enigo::Enigo;
use enigo::KeyboardControllable;
use enigo::MouseControllable;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::channel;
use rayon::prelude::*;

pub fn run(port: u16, pwd: String) {
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
    let (tx4, rx) = channel::<TcpStream>();
    if cfg!(target_os = "windows") {
        std::thread::spawn(move || {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
            stream.write_all(b"agent\n").unwrap();
            tx4.send(stream).unwrap();
        });
    }

    loop {
        match rx.recv() {
            Ok(mut stream) => {
                // Check connection validity
                let mut check = [0u8; 8];
                match stream.read_exact(&mut check) {
                    Ok(_) => {
                        println!("check {:?}", check);
                        if suc != check {
                            println!("Password error");
                            let _ = stream.write_all(&[2]);
                            continue;
                        } else {
                            println!("Succesfully accessed");
                        }
                    }
                    Err(_) => {
                        println!("Request error");
                        continue;
                    }
                }
                if let Err(_) = stream.write_all(&[1]) {
                    continue;
                }
                let ss = stream.try_clone().unwrap();
                let th1 = std::thread::spawn(move || {
                    if let Err(e) = std::panic::catch_unwind(||{
                        screen_stream(ss);
                    }) {
                        eprintln!("{:?}", e);
                    }
                });
                let th2 = std::thread::spawn(move || {
                    if let Err(e) = std::panic::catch_unwind(||{
                        event(stream);
                    }) {
                        eprintln!("{:?}", e);
                    }
                });
                th1.join().unwrap();
                th2.join().unwrap();
                println!("Break !");
            }
            Err(_) => {
                return;
            }
        }
    }
}

/**
 * Event handling
 */
fn event(mut stream: TcpStream) {
    let mut cmd = [0u8];
    let mut move_cmd = [0u8; 4];
    let mut enigo = Enigo::new();
    while let Ok(_) = stream.read_exact(&mut cmd) {
        match cmd[0] {
            common::KEY_UP => {
                stream.read_exact(&mut cmd).unwrap();
                if let Some(key) = key_mouse::key_to_enigo(cmd[0]) {
                    enigo.key_up(key);
                }
            }
            common::KEY_DOWN => {
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

    let clen = buf.len();
    encode(clen, &mut header);
    println!("header sent: {:?}", header);
    if let Err(_) = stream.write_all(&header) {
        return;
    }
    println!("buf first: {:?} buf last: {:?} buf len: {}", buf.first(), buf.last(), buf.len());
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
        println!("header sent: {:?}", header);
        if let Err(_) = stream.write_all(&header) {
            return;
        }
        if let Err(_) = stream.write_all(&buf) {
            return;
        }
    }
}
