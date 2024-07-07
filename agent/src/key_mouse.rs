/// The function `mouse_to_enigo` maps a key code to a corresponding enigo mouse button.
/// 
/// Arguments:
/// 
/// * `key`: The `key` parameter in the `mouse_to_enigo` function is of type `u8`, which represents an
/// unsigned 8-bit integer. This function takes a key value as input and returns an `Option` containing
/// an `enigo::MouseButton` based on the key value.
/// 
/// Returns:
/// 
/// The function `mouse_to_enigo` returns an `Option` containing an `enigo::MouseButton` enum value. The
/// function matches the input `key` with specific values and returns `Some(enigo::MouseButton::Left)`
/// if `key` is 233, `Some(enigo::MouseButton::Right)` if `key` is 235, and `None` for any other value
/// of
pub fn mouse_to_enigo(key: u8) -> Option<enigo::MouseButton> {
    match key {
        233 => Some(enigo::MouseButton::Left),
        235 => Some(enigo::MouseButton::Right),
        _ => None,
    }
}


/// The function `key_to_enigo` maps a given key code to the corresponding `enigo::Key` enum variant in
/// Rust.
/// 
/// Arguments:
/// 
/// * `key`: The function `key_to_enigo` takes a `u8` key code as input and returns an
/// `Option<enigo::Key>`. It matches the input key code to specific key mappings and returns the
/// corresponding `enigo::Key` if a match is found. If no match is
/// 
/// Returns:
/// 
/// The function `key_to_enigo` takes a `u8` key value and returns an `Option<enigo::Key>`. Inside the
/// function, there is a match statement that matches the input key value to specific cases and returns
/// the corresponding `enigo::Key` enum value wrapped in a `Some` variant. If the key value does not
/// match any of the specified cases, it
pub fn key_to_enigo(key: u8) -> Option<enigo::Key> {
    match key {
        27 => Some(enigo::Key::Escape),
        190 => Some(enigo::Key::F1),
        191 => Some(enigo::Key::F2),
        192 => Some(enigo::Key::F3),
        193 => Some(enigo::Key::F4),
        194 => Some(enigo::Key::F5),
        195 => Some(enigo::Key::F6),
        196 => Some(enigo::Key::F7),
        197 => Some(enigo::Key::F8),
        198 => Some(enigo::Key::F9),
        199 => Some(enigo::Key::F10),
        200 => Some(enigo::Key::F11),
        201 => Some(enigo::Key::F12),
        // 19 => Some(enigo::Key::Pause), // Pause
        // 97 => Some(enigo::Key::Print), // Print
        255 => Some(enigo::Key::Delete),
        87 => Some(enigo::Key::End),
        96 => Some(enigo::Key::Layout('`')),
        48 => Some(enigo::Key::Layout('0')),
        49 => Some(enigo::Key::Layout('1')),
        50 => Some(enigo::Key::Layout('2')),
        51 => Some(enigo::Key::Layout('3')),
        52 => Some(enigo::Key::Layout('4')),
        53 => Some(enigo::Key::Layout('5')),
        54 => Some(enigo::Key::Layout('6')),
        55 => Some(enigo::Key::Layout('7')),
        56 => Some(enigo::Key::Layout('8')),
        57 => Some(enigo::Key::Layout('9')),
        45 => Some(enigo::Key::Layout('-')),
        61 => Some(enigo::Key::Layout('=')),
        8 => Some(enigo::Key::Backspace),
        9 => Some(enigo::Key::Tab),
        91 => Some(enigo::Key::Layout('[')),
        93 => Some(enigo::Key::Layout(']')),
        92 => Some(enigo::Key::Layout('\\')),
        229 => Some(enigo::Key::CapsLock),
        59 => Some(enigo::Key::Layout(';')),
        39 => Some(enigo::Key::Layout('\'')),
        13 => Some(enigo::Key::Return),
        225 => Some(enigo::Key::Shift), // ShiftL
        44 => Some(enigo::Key::Layout(',')),
        46 => Some(enigo::Key::Layout('.')),
        47 => Some(enigo::Key::Layout('/')),
        226 => Some(enigo::Key::Shift), // ShiftR
        82 => Some(enigo::Key::UpArrow),
        227 => Some(enigo::Key::Control), // ControlL
        233 => Some(enigo::Key::Alt),     // AltL
        32 => Some(enigo::Key::Space),
        234 => Some(enigo::Key::Alt), // AltR
        // 103 => Some(enigo::Key::Menu),
        228 => Some(enigo::Key::Control), // ControlR
        81 => Some(enigo::Key::LeftArrow),
        84 => Some(enigo::Key::DownArrow),
        83 => Some(enigo::Key::RightArrow),
        // 99 => Some(enigo::Key::Raw(45)), // Insert
        86 => Some(enigo::Key::PageDown),
        80 => Some(enigo::Key::Home),
        85 => Some(enigo::Key::PageUp),
        a if a >= 97 && a <= 122 => Some(enigo::Key::Layout((a - 97 + ('a' as u8)) as char)),
        _ => None,
    }
}