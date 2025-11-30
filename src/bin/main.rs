#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![doc = include_str!("../README.md")]

use core::cell::RefCell;
use core::fmt::Write;
use heapless::String; // fixed-capacity string

use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_println::println;

// I2C
use esp_hal::i2c::master::Config as esp_I2cConfig; // for convenience, importing as alias
use esp_hal::i2c::master::I2c as esp_I2c;
use esp_hal::time::Rate;
use embedded_hal_bus::i2c::CriticalSectionDevice;

//TOF
use ::vl6180x::vl6180x::vl6180x::VL6180X;

//Oled
use tiny_oled::oled::oled::Oled;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

/// Simple helper function to scan the I2c bus for available devices and output them to stdout.
fn scan_i2c<I2C>(mut i2c: I2C) 
where 
    I2C: embedded_hal::i2c::I2c<u8>,
{
    for addr in 0x08..=0x77 {
        let mut buffer = [0u8; 1];
        match i2c.write_read(addr, &[], &mut buffer) {
            Ok(_) => println!("✓ Device found at 0x{:02X}", addr),
            Err(_) => {}
        }
    }
}
/// Main loop
#[main]
fn main() -> ! {
    // generator version: 1.0.1

    // Create I2C Bus driver instance
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());
  
    let i2c_bus: esp_I2c<'_, esp_hal::Async> = esp_I2c::new(
        peripherals.I2C0,
        // I2cConfig is alias of esp_hal::i2c::master::I2c::Config
        esp_I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_scl(peripherals.GPIO6)
    .with_sda(peripherals.GPIO5)
    .into_async();

    //Create a mutex protected shared I2C bus for use with different device drivers.
    let shared_i2c_bus = critical_section::Mutex::new(RefCell::new(i2c_bus));

    // Create and initialize OLED display     
    let mut display_proxy = CriticalSectionDevice::new(&shared_i2c_bus);
    scan_i2c(&mut display_proxy); //Output detected i2c devices to stdout
    let mut oled = Oled::new(&mut display_proxy);
    oled.init();

    // Create and initialize VL6180X sensor
    let mut vl6180x_proxy = CriticalSectionDevice::new(&shared_i2c_bus);
    let mut tof_i2c: VL6180X<&mut CriticalSectionDevice<'_, esp_I2c<'_, esp_hal::Async>>>;

    match VL6180X::new(&mut vl6180x_proxy) {
        Ok(sensor) => { tof_i2c = sensor; },
        Err(_) => {
            println!("VL6180X initialization failed");
            //TODO: Proper error handling on hardware problems
            loop {}
        }
    }
    // Create a stack-allocated string with capacity 32 bytes for outputting of measured values to the oled display
    let mut distance_str: String<32> = String::new();
    distance_str.clear();
    match write!(distance_str, "Starting") {
        Ok(_) => {oled.draw_text(&distance_str);},
        Err(_) => println!("String write error"),
    }
    
    //Check the model Id of the TOF sensor
    match tof_i2c.get_model_id() {
        Ok(r) => {
            if r != 0xb4 {
                println!("VL6180X unexpected model id 0x{:x}, check wiring!",r);
                loop {} //FIXME proper error handling on hardware issues
            }
            else {
                println!("VL6180X found correct model Id 0x{:x}!",r);
            }
        }
        Err(_) => {
            println!("VL6180X get model Id failed");
            loop {} //FIXME proper error handling on hardware issues
        }
    }

    //Main loop starts here
    loop {
        let mut update_display= false;
        led.toggle();

        match tof_i2c.start_ranging() {
            Ok(_) => {} //println!("VL6180X ranging started"),
            Err(_) => {
                println!("VL6180X ranging start failed");
                loop {} //FIXME proper error handling
            }
        }

        distance_str.clear();

        match tof_i2c.read_range() {
            Ok(r) => {
                if r != 255 {
                    match write!(distance_str, "{} mm", r) {
                        Ok(_) => {update_display = true;},
                        Err(_) => println!("String write error"),
                    }
                }
                else {
                    match write!(distance_str, "OUT mm") {
                        Ok(_) => {update_display = true;},
                        Err(_) => println!("String write error"),
                    }
                }
            },
            Err(_) => {
                println!("Range read error");
                match write!(distance_str, " Err") {
                    Ok(_) => {update_display = true;},
                    Err(_) => println!("String write error"),
                }
            },
        };
 
        if update_display
        {
            oled.draw_text(&distance_str);
        }

        //Delay loop before next measurement cycle
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(50) {}
    }
}
