// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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

    // defalut port
    let mut port = 8080;
    if args.len() >= 3 {
        port = args[2].parse::<u16>().unwrap();
    }

    // run forever
    agent::run(port, pwd);
}
