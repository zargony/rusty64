//!
//! Display/screen interface
//!

// FIXME: UI is hard to test (and optional), so there are no tests yet
#![allow(dead_code)]

use std::slice;
use super::sdl2::{pixels, render, video};

/// A screen is a graphics window presented to the user
pub struct Screen {
    pub width: uint,
    pub height: uint,
    renderer: ~render::Renderer,
    texture: ~render::Texture,
    buffer: ~[u32],
}

impl Screen {
    /// Create a new screen with the given width and height
    pub fn new (title: &str, width: uint, height: uint) -> Screen {
        let flags = [video::Shown, video::Resizable];
        let window = match video::Window::new(title, video::PosUndefined, video::PosUndefined, width as int, height as int, flags) {
            Ok(window) => window,
            Err(err) => fail!("ui: Failed to create SDL2 window: {}", err),
        };
        let flags = [render::Accelerated];
        let renderer = match render::Renderer::from_window(window, render::DriverAuto, flags) {
            Ok(renderer) => renderer,
            Err(err) => fail!("ui: Failed to create SDL2 renderer: {}", err),
        };
        let texture = match renderer.create_texture(pixels::ARGB8888, render::AccessStreaming, width as int, height as int) {
            Ok(texture) => texture,
            Err(err) => fail!("ui: Failed to create SDL2 texture: {}", err),
        };
        let buffer = slice::from_elem(width * height, 0u32);
        Screen { width: width, height: height, renderer: renderer, texture: texture, buffer: buffer }
    }

    /// Returns a reference to the screen buffer (a vector of width*height ARGB values)
    pub fn buffer<'a> (&'a mut self) -> &'a mut [u32] {
        // FIXME: If rust-sdl2 had support for SDL_LockTexture, we could use the texture buffer directly
        self.buffer.as_mut_slice()
    }

    /// Clear the screen buffer using the given value
    pub fn clear (&mut self, value: u32) {
        for pixel in self.buffer.mut_iter() {
            *pixel = value;
        }
    }

    /// Presents the current screen buffer to the user
    pub fn present (&mut self) {
        // Update the texture with the contents of the screen buffer
        unsafe { slice::raw::buf_as_slice(self.buffer.as_ptr() as *u8, 4 * self.buffer.len(), |bytes| {
            self.texture.update(None, bytes, 4 * self.width as int);
        }); }
        // Render the texture (stretching it to fill the render context)
        self.renderer.copy(self.texture, None, None);
        // Present the rendered content to the user
        self.renderer.present();
    }
}
