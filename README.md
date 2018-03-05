
# Rust Examples

These are mainly short programs I've written for myself by following
some blog post or examing some piece of code. Maybe this is also
usefull for for others.  

Feel free to suggest changes or use the code for whatever you like.

# Framebuffer

This is a small example on how-to utilize the framebuffer from
rust. The application is modeled after this well written [blog post](frame)
and without proper error handling or being rustonomic.

It uses three crates to achive the same functionality.
The `chrono` crate for easy time handling, `flate2` for gzip support
(the fonts are compressed and the used zlib library doesn't work well through ffi)
and `libc` for better ffi handling.

# Type checker

A small type checker written after this [blog post](type). It models
the basic function and sum type which can be used to type check some basic
terms. The idea is to take a term (mostly constructed from an ast) and 
a provided type (no type inference) and check if they match.
A context can be used for more complex terms and if variable names are generated
but this is not covered. 

At the moment some comments are missing and the code is not very elegant
modeled along-side the rust type system. This can be improved by using enums for
types and contexts (maybe i have some spare time).

[frame]: https://cmcenroe.me/2018/01/30/fbclock.html
[type]: http://languagengine.co/blog/so-you-want-to-write-a-type-checker/
