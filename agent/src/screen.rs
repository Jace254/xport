use scrap::Capturer;
use scrap::Display;
use std::io::ErrorKind::WouldBlock;
use std::slice::from_raw_parts;
use std::time::Duration;

/**
 * Screenshot
 */
pub struct Cap {
    w: usize,
    h: usize,
    capturer: Option<Capturer>,
    sleep: Duration,
}

unsafe impl Send for Cap {}

impl Cap {
    /// The function `new` creates a new `Cap` instance with a display and capturer based on the platform.
    ///
    /// Returns:
    ///
    /// A `Cap` struct is being returned from the `new` function.
    pub fn new() -> Cap {
        let display = Display::primary().unwrap();
        #[cfg(windows)]
        let capturer = Capturer::new(display, true).unwrap();
        #[cfg(not(windows))]
        let capturer = Capturer::new(display).unwrap();
        let (w, h) = (capturer.width(), capturer.height());
        Cap {
            w,
            h,
            capturer: Some(capturer),
            sleep: Duration::new(1, 0) / 24,
        }
    }
    fn reload(&mut self) {
        println!("Reload capturer");
        drop(self.capturer.take());
        let display = match Display::primary() {
            Ok(display) => display,
            Err(_) => {
                return;
            }
        };

        #[cfg(windows)]
        let capturer = match Capturer::new(display, true) {
            Ok(capturer) => capturer,
            Err(_) => return,
        };
        #[cfg(not(windows))]
        let capturer = match Capturer::new(display) {
            Ok(capturer) => capturer,
            Err(_) => return,
        };
        self.capturer = Some(capturer);
    }
    /// The function `wh` in Rust returns a tuple containing the width and height of a struct instance.
    pub fn wh(&self) -> (usize, usize) {
        (self.w, self.h)
    }
    #[inline]
    pub fn cap(&mut self) -> &[u8] {
        loop {
            match &mut self.capturer {
                Some(capturer) => {
                    // Wait until there's a frame.
                    let cp = capturer.frame();
                    let buffer = match cp {
                        Ok(buffer) => buffer,
                        Err(error) => {
                            std::thread::sleep(self.sleep);
                            if error.kind() == WouldBlock {
                                // Keep spinning.
                                continue;
                            } else {
                                std::thread::sleep(std::time::Duration::from_millis(200));
                                self.reload();
                                continue;
                            }
                        }
                    };
                    return unsafe { from_raw_parts(buffer.as_ptr(), buffer.len()) };
                }
                None => {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    self.reload();
                    continue;
                }
            };
        }
    }
}
