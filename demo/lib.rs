const FREQ_CONST: i32 = 200; // Const to set freq of signal
use crate::config_parse::esc_config_parser;
use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Address, Channel, Pca9685};

fn check_driver_config_check(driver: String, amount: u8) {
    if !amount == 4 || !driver.eq("pca9685") {
        panic!("Incorrect module in config");
    }
}

pub fn external_pwm_prepare() -> Pca9685<I2cdev> {
    //port: String, amount: u8, driver: String
    let value = esc_config_parser();
    check_driver_config_check(value.driver, value.amount);
    let dev = I2cdev::new(value.port).unwrap();
    let address = Address::default();
    let mut pwm = Pca9685::new(dev, address).unwrap();
    pwm.set_prescale(FREQ_CONST as u8).unwrap();
    pwm.enable().expect("Error");
    return pwm;
}

pub fn get_esc_verison() -> &'static str {
    return "ESC PWM  MODULE VERSION 0.0.1 18/09/2020";
}
pub fn map(x: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> i32 {
    let val = (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
    val
}

pub fn set_throttle_external_pwm(ch1: u16, ch2: u16, ch3: u16, ch4: u16) {
    let mut i2c_controller;
    i2c_controller = external_pwm_prepare();

    i2c_controller
        .set_channel_on(Channel::C0, map(ch1 as i32, 1000, 2000, 0, 4095) as u16)
        .unwrap();
    i2c_controller
        .set_channel_on(Channel::C1, map(ch2 as i32, 1000, 2000, 0, 4095) as u16)
        .unwrap();
    i2c_controller
        .set_channel_on(Channel::C2, map(ch3 as i32, 1000, 2000, 0, 4095) as u16)
        .unwrap();
    i2c_controller
        .set_channel_on(Channel::C3, map(ch4 as i32, 1000, 2000, 0, 4095) as u16)
        .unwrap();
    print!("{}",ch3 );
}
