#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![doc = include_str!("./README.md")]

///
/// Tiny Oled display abstraction
/// 
/// Provides simple one line text output capabilities on a 0,42" Oled display using ssd1306 driver
/// Author: Fabian Tietz <fabi.303.ft@gmail.com>
/// 2025
/// 
/// 
pub mod oled;
