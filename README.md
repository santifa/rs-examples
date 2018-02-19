
# Rust Examples

These are mainly short programs I've written for myself by following
some blog post or examing some piece of code. Maybe this is also
usefull for for others.  

Feel free to suggest changes or use the code for whatever you like.

# Rust Framebuffer

This is a small example on how-to utilize the framebuffer from
rust. The application is modeled after this well written [blog post](bp)
and without proper error handling or being rustonomic.

It uses three crates to achive the same functionality.
The `chrono` crate for easy time handling, `flate2` for gzip support
(the fonts are compressed and the used zlib library doesn't work well through ffi)
and `libc` for better ffi handling.

[bp]: https://cmcenroe.me/2018/01/30/fbclock.html
