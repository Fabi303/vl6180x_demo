///
/// Tiny Oled display abstraction
/// 
/// Provides simple one line text output capabilities on a 0,42" Oled display using ssd1306 driver
/// Author: Fabian Tietz <fabi.303.ft@gmail.com>
/// 2025
/// 
pub mod oled {

// use ssd1306 display driver
use ssd1306::{prelude::*, Ssd1306};

// use embedded graphics
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_9X15},
    pixelcolor::BinaryColor,
    prelude::{Point, *},
    text::{Baseline, Text},
};

#[allow(dead_code)]
/// Default I2C address for the Oled display
const DEFAULT_ADDRESS: u8 = 0x3C;

/// Oled display struct
pub struct Oled<I2C> {
    bus:  I2C,
    address:u8,
    display_offset_x:i32,
    display_offset_y:i32,
    display_width:u32,
    display_height:u32,
}

/// Oled display implementation
/// Uses embedded-hal I2C traits
impl<I2C> Oled<I2C>
where
    I2C: embedded_hal::i2c::I2c<u8>,
{
    /// Create new Oled display instance using default address
    pub fn new(i2c: I2C) -> Self {
        Self::with_address(i2c, DEFAULT_ADDRESS)
    }

    /// Create new Oled display instance using custom address
    pub fn with_address(i2c:I2C, address: u8) -> Self {
        let _byte:u8 = 0;

        let mut oled = Oled{
            bus:i2c, 
            address:address,
            display_offset_x: 30,
            display_offset_y: 26,
            display_width: 70,
            display_height: 38,
        };
        oled.init();
        oled
    }

    /// Initialize the Oled display
    pub fn init(&mut self) -> () {

        let _byte:u8 = 0;
        let i2c_interface = I2CInterface::new(&mut self.bus, self.address, _byte);
        let mut display = Ssd1306::new(i2c_interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();

        let _ = display.init();
        let _ = display.flush();
    }

    /// Draw text on the Oled display
    pub fn draw_text(&mut self, text: &str) -> () {
        let _byte:u8 = 0x40; //Required for setting a custom address
        let i2c_interface = I2CInterface::new(&mut self.bus, self.address, _byte);
        let mut display: Ssd1306<I2CInterface<&mut I2C>, DisplaySize128x64, ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>> = Ssd1306::new(i2c_interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();
        //display.init();
        display.clear_buffer();


        let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X15)
        .text_color(BinaryColor::On)
        .build();
    
        //TODO: Center text properly
        let _ = Text::with_baseline(text, Point::new(self.display_offset_x-28+(self.display_width/2) as i32, self.display_offset_y-6+(self.display_height/2) as i32), text_style, Baseline::Top)
        .draw(&mut display);
        let _ = display.flush();
    }


}
}   //mod oled
  