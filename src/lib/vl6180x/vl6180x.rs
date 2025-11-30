///
/// Very simple VL6180x driver using embedded-hal I2C traits
/// currently supports basic initialization and distance measurement
/// 
/// WARNING:
/// This driver is for demonstration purposes only, no proper error handling for on chip error conditions
/// is implemented yet.
/// 
/// 
/// Author: Fabian Tietz <fabi.303.ft@gmail.com>
/// 2025
/// 
pub mod vl6180x {

use embedded_hal::i2c::I2c;
use crate::vl6180x_regs::vl6180x_regs::Register;


#[allow(dead_code)]
const MODEL_ID: u8 = 0xB4;
const DEFAULT_ADDRESS: u8 = 0x29;

/// VL6180X sensor struct
pub struct VL6180X<I2C> {
    bus: I2C,
    address: u8
}

/// VL6180X sensor implementation
impl<I2C:I2c> VL6180X<I2C>
where
    I2C: embedded_hal::i2c::I2c<u8>,
{
    /// Create new VL6180X instance using default address
    pub fn new(i2c: I2C) -> Result<Self, I2C::Error> {
        Self::with_address(i2c, DEFAULT_ADDRESS)
    }

    /// Create new VL6180X instance using custom address
    pub fn with_address(i2c:I2C, address: u8) -> Result<Self, I2C::Error> {
        let mut vl = VL6180X{bus:i2c, address:address};
       
        match vl.who_am_i() {
            Ok(res) => {
                if res == MODEL_ID {
                    // We got the correct model id from the device
                    match vl.init() {
                        Ok(_) => {
                            Ok(vl)
                        }
                        Err(_) => {
                            loop {} //Fixme: Proper error handling
                        }
                    }
                    
                }
                else {
                    //create custom error, because the I2c transfer worked ok, but the device returned invalid data
                    //It is not possible to instantiate I2c hal errors (no new function available).
                    //So we have to define our own error type in the future
                    loop {}
                }
                
            },
            Err(e)=> {
                Err(e)
            }
        }
    }

    /// Read the Model ID register to verify device identity
    fn who_am_i(&mut self) -> Result<u8, I2C::Error> {
        self.read_reg(Register::IDENTIFICATION__MODEL_ID)
    }

    /// Initialize the VL6180X sensor
    pub fn init(&mut self) -> Result<(), I2C::Error> {
        match self.read_reg(Register::SYSTEM__FRESH_OUT_OF_RESET) {
            Ok(res) => {
                if res == 0x01 {
                   
                    self.load_recommended_config()?;
                    self.write_reg(Register::SYSTEM__FRESH_OUT_OF_RESET, 0x00)?
                }
                
            },
            Err(_e)=> {}
        }
        Ok(())
    }

    /// Start a single ranging measurement
    pub fn start_ranging(&mut self) -> Result<(), I2C::Error> {
        self.write_reg(Register::SYSRANGE__START, 0x01)
    }

    /// Clear any pending interrupts
    pub fn clear_int(&mut self) -> Result<(), I2C::Error> {
        self.write_reg(Register::SYSTEM__INTERRUPT_CLEAR, 0x07)
    }

    /// Read the measured range in mm from RESULT__RANGE_VAL register
    pub fn read_range(&mut self) -> Result<u8, I2C::Error> {
        self.read_reg(Register::RESULT__RANGE_VAL)
    }

    /// Read the interrupt status from RESULT__INTERRUPT_STATUS_GPIO register
    pub fn int_status(&mut self) -> Result<u8, I2C::Error> {
        self.read_reg(Register::RESULT__INTERRUPT_STATUS_GPIO)
    }

    /// Read a byte from a register
    fn read_reg(&mut self, reg: Register) -> Result<u8, I2C::Error> {
        let mut buffer: [u8; 1] = [0];
        match self.bus.write_read(self.address, &reg.addr(), &mut buffer) {
            Ok(_) => Ok(buffer[0]),
            Err(e) => return Err(e),
        }
    }

    /// Write a byte to a register
    fn write_reg(&mut self, reg: Register, value: u8) -> Result<(), I2C::Error> {
        let register = reg.addr();
        let bytes = [register[0], register[1], value];
        match self.bus.write(self.address, &bytes) {
            Ok(_) => {Ok(())},
            Err(e) => return Err(e),
        }
    }

    /// Load the recommended configuration settings into the sensor
    /// Taken from VL6180X datasheet and application notes
    fn load_recommended_config(&mut self) -> Result<(), I2C::Error> {
        // Mandatory : private registers, values taken from the datasheet
        self.write_byte(0x0207, 0x01)?;
        self.write_byte(0x0208, 0x01)?;
        self.write_byte(0x0096, 0x00)?;
        self.write_byte(0x0097, 0xfd)?;
        self.write_byte(0x00e3, 0x01)?;
        self.write_byte(0x00e4, 0x03)?;
        self.write_byte(0x00e5, 0x02)?;
        self.write_byte(0x00e6, 0x01)?;
        self.write_byte(0x00e7, 0x03)?;
        self.write_byte(0x00f5, 0x02)?;
        self.write_byte(0x00d9, 0x05)?;
        self.write_byte(0x00db, 0xce)?;
        self.write_byte(0x00dc, 0x03)?;
        self.write_byte(0x00dd, 0xf8)?;
        self.write_byte(0x009f, 0x00)?;
        self.write_byte(0x00a3, 0x3c)?;
        self.write_byte(0x00b7, 0x00)?;
        self.write_byte(0x00bb, 0x3c)?;
        self.write_byte(0x00b2, 0x09)?;
        self.write_byte(0x00ca, 0x09)?;
        self.write_byte(0x0198, 0x01)?;
        self.write_byte(0x01b0, 0x17)?;
        self.write_byte(0x01ad, 0x00)?;
        self.write_byte(0x00ff, 0x05)?;
        self.write_byte(0x0100, 0x05)?;
        self.write_byte(0x0199, 0x05)?;
        self.write_byte(0x01a6, 0x1b)?;
        self.write_byte(0x01ac, 0x3e)?;
        self.write_byte(0x01a7, 0x1f)?;
        self.write_byte(0x0030, 0x00)?;

        // Recommended : Public registers - See data sheet for more detail
        // Enables polling for ‘New Sample ready, when measurement is complete
        self.write_byte(0x0011, 0x10)?;   

        // set averaging sample period
        self.write_byte(0x010a, 0x30)?;
        // Sets the light and dark gain (upper nibble), dark gain should not be changed.
        self.write_byte(0x0031, 0xFF)?; 

        // Set ALS integration time to 100ms
        self.write_byte(0x0041, 0x63)?;   // Set ALS integration time to 100ms

        //perform a single temperature calibration of the ranging sensor
        self.write_byte(0x002e, 0x01)?;   

        // Optional: Public registers - See data sheet for more detail

        //Set default ranging inter-measurement period to 100ms
        self.write_byte(0x001b, 0x09)?;   

        //Set default ALS inter-measurement period to 500ms                                 
        self.write_byte(0x003e, 0x31)?;

        //Configure interrupt on new available sample
        self.write_byte(0x0014, 0x24)?;

        Ok(())
    }

    /// Write byte to register
    fn write_byte(&mut self, reg: u16, byte: u8) -> Result<(), I2C::Error> {
        let bytes = [((reg & 0xFF00) >> 8) as u8, (reg & 0x00FF) as u8, byte];
        match self.bus.write(self.address, &bytes) {
            Ok(_) => {Ok(())},
            Err(e) => return Err(e),
        }
    }


    /// Returns the model ID from IDENTIFICATION__MODEL_ID register.
    /// This value is expected to be 0xB4
    pub fn get_model_id(&mut self) -> Result<u8, I2C::Error> {
        let id = self.read_reg(Register::IDENTIFICATION__MODEL_ID)?;
        Ok(id)
    }

}
}