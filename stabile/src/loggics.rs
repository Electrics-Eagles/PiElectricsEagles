const START_MOTOS_VALUE: u16 = 10;

use crate::mpu6050::Mpu6050_driver;
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::Mpu6050;
extern crate pid;
use pid::Pid;
use pwm_pca9685::{Address, Channel, Pca9685};
use std::{
    thread,
    time::{self, SystemTime},
};

use crate::config_parse::*;

use crate::clk_driver::ClkDriver;
use crate::controller::*;
use crate::ibus::*;
use crate::mpu6050::*;
use crate::simple_logger;

fn convert(v: u8) -> f64 {
    return v as f64;
}

pub fn main_loop() {
    let mut loops = 0;
    let mut reciver_driver = ibus_receiver::new();
    let mut mpu6050 = Mpu6050_driver::new();
    let mut controller = Controller::new();
    let mut config = config_parser::new();
    let mut clk_driver = ClkDriver::new();

    simple_logger::logger(1, true, "CREATE DRIVER OBJECTS :".parse().unwrap());

    /* init*/
    loop {
        clk_driver.set_pin_clk_high();
        let now = SystemTime::now();
        let reciver = reciver_driver.get_datas_of_channel_form_ibus_receiver();
        simple_logger::logger(1, true, "READ DATA FROM RC :".parse().unwrap());
        let autolevel = config.auto_level_config();
        simple_logger::logger(1, true, "LIST SETTINGS :".parse().unwrap());
        simple_logger::logger(1, true, autolevel.to_string().parse().unwrap());

        let pid_settings = config.get_pids();
        let mut pid_roll = Pid::new(
            pid_settings.roll.p as f64,
            pid_settings.roll.i as f64,
            pid_settings.roll.d as f64,
            pid_settings.roll.max as f64,
            pid_settings.roll.max as f64,
            0.0,
            0.0,
        );
        print!("{}", reciver.ch6);
        let mut pid_pitch = Pid::new(
            pid_settings.pitch.p as f64,
            pid_settings.pitch.i as f64,
            pid_settings.pitch.d as f64,
            pid_settings.pitch.max as f64,
            pid_settings.pitch.max as f64,
            0.0,
            0.0,
        );
        let mut pid_yaw = Pid::new(
            pid_settings.yaw.p as f64,
            pid_settings.yaw.i as f64,
            pid_settings.yaw.d as f64,
            pid_settings.yaw.max as f64,
            pid_settings.yaw.max as f64,
            0.0,
            0.0,
        );

        let acc_value = mpu6050.get_acc_values(1);

        let acc_x = acc_value.x;
        let acc_y = acc_value.y;
        let acc_z = acc_value.z;

        let acc_total_vector_no_square = (acc_x.pow(2) + acc_y.pow(2) + acc_z.pow(2)) as f64;
        let acc_total_vector: f64 = acc_total_vector_no_square.sqrt();

        simple_logger::logger(1, true, "acc_total_vector_no_square".to_string());
        simple_logger::logger(
            1,
            true,
            acc_total_vector_no_square.to_string().parse().unwrap(),
        );
        simple_logger::logger(1, true, "acc_total_vector".to_string());
        simple_logger::logger(1, true, acc_total_vector.to_string().parse().unwrap());
        let mut angle_pitch_acc: f64 = 0.0;
        let mut angle_roll_acc: f64 = 0.0;
        let mut angle_pitch: f64 = 0.0;
        let mut angle_roll: f64 = 0.0;
        let mut pitch_level_correction;
        let mut roll_level_correction;
        let mut start: i32 = 0;
        let gyro_values = mpu6050.get_gyro_values(1);
        let mut throllite;
        let mut esc_1;
        let mut esc_2;
        let mut esc_3;
        let mut esc_4;

        simple_logger::logger(1, true, "start".to_string());
        simple_logger::logger(1, true, start.to_string().parse().unwrap());

        angle_pitch += convert(acc_x) * 0.0000611; //Calculate the traveled pitch angle and add this to the angle_pitch variable.
        angle_roll += convert(acc_z) * 0.0000611;

        simple_logger::logger(1, true, "acc_z".to_string());
        simple_logger::logger(1, true, acc_z.to_string().parse().unwrap());

        simple_logger::logger(1, true, "acc_x".to_string());
        simple_logger::logger(1, true, acc_x.to_string().parse().unwrap());

        if convert(acc_y).abs() < acc_total_vector {
            angle_pitch_acc = (convert(acc_y) / acc_total_vector).asin() * 57.296;
        }

        simple_logger::logger(1, true, "angle_pitch_acc".to_string());
        simple_logger::logger(1, true, angle_pitch_acc.to_string().parse().unwrap());
        if convert(acc_x).abs() < acc_total_vector {
            angle_roll_acc = (convert(acc_x) / acc_total_vector).asin() * -57.296;
        }

        simple_logger::logger(1, true, "angle_roll_acc".to_string());
        simple_logger::logger(1, true, angle_roll_acc.to_string().parse().unwrap());
        angle_pitch_acc -= 0.0;
        angle_roll_acc -= 0.0;

        angle_pitch = angle_pitch * 0.9996 + angle_pitch_acc * 0.0004; //Correct the drift of the gyro pitch angle with the accelerometer pitch angle.
        angle_roll = angle_roll * 0.9996 + angle_roll_acc * 0.0004; //Correct the drift of the gyro roll angle with the accelerometer roll angle.

        simple_logger::logger(1, true, "angle_pitch".to_string());
        simple_logger::logger(1, true, angle_pitch.to_string().parse().unwrap());
        simple_logger::logger(1, true, "angle_roll_acc".to_string());
        simple_logger::logger(1, true, angle_roll_acc.to_string().parse().unwrap());
        pitch_level_correction = angle_pitch * 15 as f64; //Calculate the pitch angle correction
        roll_level_correction = angle_roll * 15 as f64; //Calculate the roll angle correction
        simple_logger::logger(1, true, "angle_pitch".to_string());
        simple_logger::logger(1, true, angle_pitch.to_string().parse().unwrap());
        simple_logger::logger(1, true, "angle_roll".to_string());
        simple_logger::logger(1, true, angle_roll.to_string().parse().unwrap());
        if autolevel == 0 {
            //If the quadcopter is not in auto-level mode
            pitch_level_correction = 0.0; //Set the pitch angle correction to zero.
            roll_level_correction = 0.0; //Set the roll angle correcion to zero.
        }

        loops = loops + 1;

        simple_logger::logger(1, true, "loops".to_string());
        simple_logger::logger(1, true, loops.to_string().parse().unwrap());

        if reciver.ch6 > 1900 {
            /*
            controller.turn_motor(Channel::C0, 1000);
            controller.turn_motor(Channel::C1, 1000);
            controller.turn_motor(Channel::C2, 1000);
            controller.turn_motor(Channel::C3, 1000);
            */
            start = 2;
            angle_pitch = angle_pitch_acc; //Set the gyro pitch angle equal to the accelerometer pitch angle when the quadcopter is started.
            angle_roll = angle_roll_acc;

            simple_logger::logger(1, true, "angle_pitch_acc".to_string());
            simple_logger::logger(1, true, angle_pitch_acc.to_string().parse().unwrap());

            simple_logger::logger(1, true, "angle_roll_acc".to_string());
            simple_logger::logger(1, true, angle_roll_acc.to_string().parse().unwrap());

            print!("{}", "Unlocked 1");
        }

        if start == 2 && reciver.ch6 < 1050 {
            start = 0;
            print!("{}", "Blocked 3");
        }

        pid_roll.setpoint = 0.0;
        //We need a little dead band of 16us for better results.
        if reciver.ch1 > 1508 {
            pid_roll.setpoint = reciver.ch1 as f64 - 1508.0;
        } else if reciver.ch1 < 1492 {
            pid_roll.setpoint = reciver.ch1 as f64 - 1492.0;
        }

        pid_roll.setpoint -= roll_level_correction; //Subtract the angle correction from the standardized receiver roll input value.
        pid_roll.setpoint /= 3.0;
        simple_logger::logger(1, true, "pid_roll.setpoint".to_string());
        simple_logger::logger(1, true, pid_roll.setpoint.to_string().parse().unwrap());
        pid_pitch.setpoint = 0.0;
        //We need a little dead band of 16us for better results.
        if reciver.ch2 > 1508 {
            pid_pitch.setpoint = reciver.ch2 as f64 - 1508.0;
        } else if reciver.ch2 < 1492 {
            pid_pitch.setpoint = reciver.ch2 as f64 - 1492.0;
        }

        pid_pitch.setpoint -= pitch_level_correction; //Subtract the angle correction from the standardized receiver pitch input value.
        pid_pitch.setpoint /= 3.0;
        simple_logger::logger(1, true, "pid_pitch.setpoint".to_string());
        simple_logger::logger(1, true, pid_pitch.setpoint.to_string().parse().unwrap());
        pid_yaw.setpoint = 0.0;
        //We need a little dead band of 16us for better results.
        if reciver.ch3 > 1050 {
            //Do not yaw when turning off the motors.
            if reciver.ch4 > 1508 {
                pid_yaw.setpoint = (reciver.ch4 as f64 - 1508.0) / 3.0 as f64;
            } else if reciver.ch4 < 1492 {
                pid_yaw.setpoint = (reciver.ch4 as f64 - 1492.0) / 3.0 as f64;
            }
        }
        let pid_output_roll = pid_roll
            .next_control_output(gyro_values.x as f64 - pid_roll.setpoint)
            .output;

        simple_logger::logger(1, true, "pid_output_roll".to_string());
        simple_logger::logger(1, true, pid_output_roll.to_string().parse().unwrap());
        let pid_output_pitch = pid_pitch
            .next_control_output(gyro_values.y as f64 - pid_pitch.setpoint)
            .output;
        let pid_output_yaw = pid_yaw
            .next_control_output(gyro_values.z as f64 - pid_yaw.setpoint)
            .output;

        simple_logger::logger(1, true, "pid_output_pitch".to_string());
        simple_logger::logger(1, true, pid_output_pitch.to_string().parse().unwrap());

        simple_logger::logger(1, true, "pid_output_yaw".to_string());
        simple_logger::logger(1, true, pid_output_yaw.to_string().parse().unwrap());
        throllite = reciver.ch3;
        simple_logger::logger(1, true, "throllite".to_string());
        simple_logger::logger(1, true, throllite.to_string().parse().unwrap());
        if start == 2 {
            if throllite > 1800 {
                throllite = 1800;
            }
            esc_1 = throllite as f64 - pid_output_pitch + pid_output_roll - pid_output_yaw; //Calculate the pulse for esc 1 (front-right - CCW)
            esc_2 = throllite as f64 + pid_output_pitch + pid_output_roll + pid_output_yaw; //Calculate the pulse for esc 2 (rear-right - CW)
            esc_3 = throllite as f64 + pid_output_pitch - pid_output_roll - pid_output_yaw; //Calculate the pulse for esc 3 (rear-left - CCW)
            esc_4 = throllite as f64 - pid_output_pitch - pid_output_roll + pid_output_yaw; //Calculate the pulse for esc 4 (front-left - CW)
            simple_logger::logger(1, true, "esc_1".to_string());
            simple_logger::logger(1, true, esc_1.to_string().parse().unwrap());
            simple_logger::logger(1, true, "esc_2".to_string());
            simple_logger::logger(1, true, esc_2.to_string().parse().unwrap());
            simple_logger::logger(1, true, "esc_3".to_string());
            simple_logger::logger(1, true, esc_3.to_string().parse().unwrap());
            simple_logger::logger(1, true, "esc_4".to_string());
            simple_logger::logger(1, true, esc_4.to_string().parse().unwrap());

            if esc_1 < 1100.0 {
                esc_1 = 1100.0;
            } //Keep the motors running.
            if esc_2 < 1100.0 {
                esc_2 = 1100.0;
            } //Keep the motors running.
            if esc_3 < 1100.0 {
                esc_3 = 1100.0;
            } //Keep the motors running.
            if esc_4 < 1100.0 {
                esc_4 = 1100.0;
            } //Keep the motors running.

            if esc_1 > 2000.0 {
                esc_1 = 2000.0;
            } //Limit the esc-1 pulse to 2000us.
            if esc_2 > 2000.0 {
                esc_2 = 2000.0;
            } //Limit the esc-2 pulse to 2000us.
            if esc_3 > 2000.0 {
                esc_3 = 2000.0;
            } //Limit the esc-3 pulse to 2000us.
            if esc_4 > 2000.0 {
                esc_4 = 2000.0;
            }
        } else {
            esc_1 = 1000.0; //If start is not 2 keep a 1000us pulse for ess-1.
            esc_2 = 1000.0; //If start is not 2 keep a 1000us pulse for ess-2.
            esc_3 = 1000.0; //If start is not 2 keep a 1000us pulse for ess-3.
            esc_4 = 1000.0; //If start is not 2 keep a 1000us pulse for ess-4.
        }

        simple_logger::logger(1, true, "esc_1".to_string());
        simple_logger::logger(1, true, esc_1.to_string().parse().unwrap());
        simple_logger::logger(1, true, "esc_2".to_string());
        simple_logger::logger(1, true, esc_2.to_string().parse().unwrap());
        simple_logger::logger(1, true, "esc_3".to_string());
        simple_logger::logger(1, true, esc_3.to_string().parse().unwrap());
        simple_logger::logger(1, true, "esc_4".to_string());
        simple_logger::logger(1, true, esc_4.to_string().parse().unwrap());
        controller.set_throttle_external_pwm(
            esc_1 as u16,
            esc_2 as u16,
            esc_3 as u16,
            esc_4 as u16,
        );

        print!("{} \n", esc_1);
        let ten_millis = time::Duration::from_millis(100);
        println!("{}", now.elapsed().expect("err").as_millis());
        /*
        set_throttle_external_pwm(esc_1 as u16, esc_2 as u16, esc_3 as u16, esc_4 as u16);
        controller.turn_motor(Channel::C0, esc_1 as u16);
        controller.turn_motor(Channel::C1,  esc_2 as u16);
        controller.turn_motor(Channel::C2, esc_3 as u16);
        controller.turn_motor(Channel::C3, esc_4 as u16);
        */
        clk_driver.set_pin_clk_low();
    }
}