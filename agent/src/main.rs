#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod key_mouse;
mod screen;
mod agent;
fn main() {
    let args: Vec<String> = std::env::args().collect();

    // default password
    let mut pwd = String::from("joash123");
    if args.len() >= 2 {
        pwd = args[1].clone();
    }

    // defalut host
    let mut host = String::from("127.0.0.1:8080");
    if args.len() >= 3 {
        host = args[2].clone();
    }

    // run forever
    agent::run(host, pwd);
}
