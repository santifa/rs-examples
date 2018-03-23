
# Rust Examples

These are mainly short programs I've written for myself by following
some blog post or examing some piece of code. Maybe this is also
usefull for for others.  

Run the examples with `cargo run --bin PROG` but the main methods
are mostly demonstration and do not necessarily output anything useful.
Read an play with the code to get insight on the example. If in doubt
write a test for a specific function you want to explore and run with 
`cargo test --bin PROG`. 

Feel free to suggest changes or use the code for whatever you like 
since it's under BSD 2-Clause License.

## Framebuffer

This is a small example on how-to utilize the framebuffer from
rust. The application is modeled after this well written [blog post](frame)
and without proper error handling or being rustonomic.

It uses three crates to achive the same functionality.
The `chrono` crate for easy time handling, `flate2` for gzip support
(the fonts are compressed and the used zlib library doesn't work well through ffi)
and `libc` for better ffi handling.

[frame]: https://cmcenroe.me/2018/01/30/fbclock.html

## Type checker

A small type checker written after this [blog post](type). It models
the basic function and sum type which can be used to type check some basic
terms. The idea is to take a term (mostly constructed from an ast) and 
a provided type (no type inference) and check if they match.
A context can be used for more complex terms and if variable names are generated
but this is not covered. 

At the moment some descriptions are missing and some points can be better modeled
along-side the rust type system.

[type]: http://languagengine.co/blog/so-you-want-to-write-a-type-checker/

## ML Classification

A naive gaussian bayes classifier is demonstrated [here](ml). It runs on a
glass dataset with two classifiers (glass or not). 

At the moment the near copy of python code and panda magic produces to complex functions.
It lacks some abstraction. The classifier should be a trait with default implementation
and the underlying data structure should be an associated type.

[ml]: https://www.antoniomallia.it/lets-implement-a-gaussian-naive-bayes-classifier-in-python.html
