
use crate::config_parse::config_parser;
use crate::simple_logger;
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::*;
use std::fs::File;
use std::io::prelude::*;



pub struct GyroMpu6050RawData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub struct AccMpu6050RawData {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}
pub struct Mpu6050_driver {
    value_of_gyro: Mpu6050<I2cdev, Delay>,
}
impl Mpu6050_driver {
    pub fn new() -> Mpu6050_driver {
      
        let mut config = config_parser::new();
        let mpu6050_conifg = config.mpu_config_parser();
        println!("{}", mpu6050_conifg.port);
        simple_logger::logger(1, true, "READ MPU Config".parse().unwrap());
        let i2c = I2cdev::new(mpu6050_conifg.port).expect("alert no port found");
        let delay = Delay;
        let mut mpu = Mpu6050::new(i2c, delay);
        mpu.init().unwrap();
        mpu.soft_calib(Steps(mpu6050_conifg.sample_amount))
            .expect("software calibrate fallut");
        mpu.calc_variance(Steps(mpu6050_conifg.sample_amount))
            .expect("calc variance error");
        Mpu6050_driver { value_of_gyro: mpu }
    }

    pub fn driver_mpu6050_version() -> &'static str {
        return " MPU6050 DRIVER  V0.0.1 verison is 14/11/2020 ID is: 4gQvYOdD";
    }

    pub fn get_acc_values(&mut self, steps: u8) -> AccMpu6050RawData {
         
        simple_logger::logger(1, true, "Read acc values".parse().unwrap());
        let data = AccMpu6050RawData {
            x: self.value_of_gyro.get_acc_avg(Steps(steps)).unwrap().x as u8,
            y: self.value_of_gyro.get_acc_avg(Steps(steps)).unwrap().y as u8,
            z: self.value_of_gyro.get_acc_avg(Steps(steps)).unwrap().z as u8,
        };
        simple_logger::logger(1, true, "ACC VALUE:".parse().unwrap());
        simple_logger::logger(1, true, data.x.to_string().parse().unwrap());
        simple_logger::logger(1, true, data.y.to_string().parse().unwrap());
        simple_logger::logger(1, true, data.z.to_string().parse().unwrap());
        return data;
    }
    pub fn get_gyro_values(&mut self, steps: u8) -> GyroMpu6050RawData {
        simple_logger::logger(1, true, "Read gyro values".parse().unwrap());
        let data = GyroMpu6050RawData {
            x: self.value_of_gyro.get_gyro_avg(Steps(steps)).unwrap().x as i32,
            y: self.value_of_gyro.get_gyro_avg(Steps(steps)).unwrap().y as i32,
            z: self.value_of_gyro.get_gyro_avg(Steps(steps)).unwrap().z as i32,
        };
        simple_logger::logger(1, true, "GYRO VALUE:".parse().unwrap());
        simple_logger::logger(1, true, data.x.to_string().parse().unwrap());
        simple_logger::logger(1, true, data.y.to_string().parse().unwrap());
        simple_logger::logger(1, true, data.z.to_string().parse().unwrap());
        return data;
    }
    pub fn get_temp(&mut self) -> f32 {
        simple_logger::logger(1, true, "Read temp values".parse().unwrap());
        simple_logger::logger(1, true, "GYRO VALUE:".parse().unwrap());
        simple_logger::logger(
            1,
            true,
            self.value_of_gyro
                .get_temp()
                .expect("error in fetch temp")
                .to_string()
                .parse()
                .unwrap(),
        );
        return self.value_of_gyro.get_temp().expect("error in fetch temp");
    }
    
    



}
