//!
//! User Inteface handling
//!

extern crate sdl2;

pub use self::screen::Screen;

mod screen;

/// Abstract object that can be created to initialize and access the UI
pub struct UI;

impl UI {
    /// Create an abstract UI object (initializes SDL2 until dropped)
    pub fn new () -> UI {
        match sdl2::init([sdl2::InitVideo]) {
            false => fail!("ui: Failed to initialize SDL2: {}", sdl2::get_error()),
            true => UI,
        }
    }

    /// Runs the UI loop and the given closure. Must be called from
    /// the main thread (SDL2 requirement)
    pub fn run (&mut self, f: || -> bool) {
        loop {
            match sdl2::event::poll_event() {
                sdl2::event::QuitEvent(..) => break,
                sdl2::event::KeyDownEvent(_, _, sdl2::keycode::EscapeKey, _, _) => break,
                _ => { },
            }
            if !f() { break; }
        }
    }
}

impl Drop for UI {
    fn drop (&mut self) {
        sdl2::quit();
    }
}


#[cfg(test)]
mod test {
    use super::UI;

    #[test]
    fn smoke () {
        let mut ui = UI::new();
        ui.run(|| { false });
    }
}
