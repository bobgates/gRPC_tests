#[allow(unused_imports)]
use anyhow::{Error, Result};
use chrono::prelude::*;
use log::{info, trace};
pub use measurements::{Angle, Pressure, Temperature};
use mysql::params;
use mysql::prelude::Queryable;
use mysql::*;
use phf::phf_map;
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::time::SystemTime;
use std::{thread, time};

use crate::data_transfer::{
    datetime_opt_to_rust, datetime_opt_to_sql, datetime_rust_to_sql, datetime_sql_to_rust,
    DataStruct, DatabaseName,
};
// use crate::defines::ALTITUDE_EVERY_NTH;
use crate::imu::inertial::Inertial;
use crate::imu::orientation::Orientation;
use crate::imu::vector3d::Vector3D;
use crate::sense_hat::sense_hat::SenseHat;
use crate::sequence::get_highest;
use crate::status::DeviceReporting;
pub use crate::tools::{i2ctool, utils::*};
use crate::two_hat::two_hat::TwoHat;

// #[cfg(feature = "sensehat")]

/// A shortcut for Results that can return `T` or `SenseHatError`.
// pub type ImuResult<T> = Result<T, ImuError>; //, ImuError>;

/// look after different types of IMU
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImuType {
    TwoHatType,
    SenseHatType,
    // NoImu,
    // Fake,
}

static IMU_I2CDEV: phf::Map<u8, ImuType> = phf_map! {
    0x6au8 => ImuType::SenseHatType,      // i2c address of accel and gyro of LSM9DS1
    0x76u8 => ImuType::TwoHatType,        // i2c address of MS583702 pressure sensor - BNO086 is not on I2C bus
};

/// Errors that this crate can return.
#[derive(Debug)]
pub enum ImuError {
    NotReady,
    // GenericError,
    // ImuData,
    // I2CError(LinuxI2CError),
    // LSM9DS1Error(lsm9ds1::Error),
    // Bno08xError(bno08x::Error),
    // ScreenError(sensehat_screen::error::ScreenError),
    // CharacterError(std::string::FromUtf16Error),
}

/// A collection of all the data from the IMU.
#[derive(Debug)]
pub struct ImuData {
    pub timestamp: u64,
    pub inertial: Result<Inertial, ImuError>,
    pub pressure: Option<Pressure>,
    pub temperature: Option<f64>,
    pub temp_cpu: Option<f64>,
    // pub humidity: Option<f64>,
}

pub trait ImuReturn {
    // fn get_data(&mut self) -> Result<ImuData, ImuError>;
    fn get_inertial(&mut self) -> Result<Inertial, ImuError>;
    fn get_pressure(&mut self) -> Option<Pressure>;
    fn get_temperature(&mut self) -> Option<Temperature>;
    fn get_cpu_temp(&mut self) -> Option<Temperature>;
}

impl ImuData {
    pub fn zero() -> ImuData {
        ImuData {
            timestamp: 0,
            inertial: Inertial::zero(),
            pressure: None,
            temperature: None,
            temp_cpu: None,
            // humidity: None,
        }
    }
}

/// Run a Pi utility to get the CPU core temperature
/// in degrees C. The result looks like: temp=43.2'C
pub fn get_cpu_temp() -> Option<Temperature> {
    let op = Command::new("/usr/bin/vcgencmd")
        .args(["measure_temp"])
        .output()
        .expect("failed to get cpu temp from Pi");
    let op = String::from_utf8_lossy(&op.stdout);
    let split = op.split('=');
    let c: Vec<&str> = split.collect();
    let c = c[1];
    let split = c.split('\'');
    let d: Vec<&str> = split.collect();
    let d = d[0].parse::<f64>();
    match d {
        Ok(n) => Some(Temperature::from_celsius(n)),
        Err(_e) => None,
    }
}

// pub fn get_temperature() -> Option<Temperature> {
//     Some(Temperature::from_celsius(0.0))
// }

/// Find out which imu type is connected.
/// This is based on asking the i2c bus what is connected and checking against
/// the presence of the imu units themselves.
/// The table of known IMUs is in IMU_I2CDEV
pub fn which_imu() -> Option<ImuType> {
    if i2ctool::scan() == None {
        return None;
    }
    let addrs: Vec<u8> = i2ctool::scan().unwrap();

    for (t_addr, device) in IMU_I2CDEV.entries() {
        if addrs.clone().into_iter().find(|&x| &x == t_addr).is_some() {
            return Some(*device);
        }
    }
    None
}

// The DB code doesn't understand temperature, so I'm using
// f64, which means converting in and out.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ImuRow {
    lineno: u64,
    uuid: u64,
    pitime: DateTime<Utc>,
    gps_time: Option<DateTime<Utc>>,
    sequence: u32,
    accel: Vector3D,
    gyro: Vector3D,
    mag: Vector3D,
    pose: Orientation,
    altitude: Option<f64>,
    temperature: Option<f64>,
    cpu_temp: Option<f64>,
    uploaded: bool,
    confirmed: bool,
}

/// What follows are the traits required for working with data_transfer
impl DataStruct for ImuRow {
    fn lineno(&self) -> u64 {
        self.lineno
    }

    fn create_data_line(&self) -> String {
        format!(
            "({},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}, {},{})", // Note quotes around datetime fields
            self.uuid,
            self.pitime.format("'%Y-%m-%d %H:%M:%S%.6f'"), // Note quotes around datetime fields
            match self.gps_time {
                Some(t) => t.format("'%Y-%m-%d %H:%M:%S%.3f'").to_string(), // same format as pitime
                None => "NULL".to_string(),
            },
            self.sequence,
            self.accel.x,
            self.accel.y,
            self.accel.z,
            self.gyro.x,
            self.gyro.y,
            self.gyro.z,
            self.pose.roll.as_degrees(),
            self.pose.pitch.as_degrees(),
            self.pose.yaw.as_degrees(),
            self.pose.heading_accuracy.as_degrees(),
            self.mag.x,
            self.mag.y,
            self.mag.z,
            match self.altitude {
                Some(t) => t.to_string(),
                None => "NULL".to_string(),
            },
            match self.temperature {
                Some(t) => t.to_string(),
                None => "NULL".to_string(),
            },
            match self.cpu_temp {
                Some(t) => t.to_string(),
                None => "NULL".to_string(),
            },
            false,
            false
        )
    }

    fn create_data_fields(&self) -> String {
        "uuid, pitime, gps_time, sequence, \
        x_accel, y_accel, z_accel, \
        x_gyro, y_gyro, z_gyro, \
        roll_pose, pitch_pose, yaw_pose, heading_accuracy, \
        x_mag, y_mag, z_mag, \
        altitude, temperature, temp_cpu, uploaded, confirmed"
            .to_string()
    }

    fn data_line_from_row(row: &mut Row) -> Self {
        // let now = SystemTime::now();
        // let dt_now: DateTime<Utc> = now.clone().into();

        let lineno: u64 = row.take(0).unwrap();
        let uuid: u64 = row.take(1).unwrap();
        let pitime: DateTime<Utc> = datetime_sql_to_rust(row.take(2).unwrap());
        let gps_time: Option<DateTime<Utc>> = datetime_opt_to_rust(row.take(3).unwrap());
        let sequence: u32 = row.take(4).unwrap();

        let accel = Vector3D {
            x: row.take(5).unwrap(),
            y: row.take(6).unwrap(),
            z: row.take(7).unwrap(),
        };

        let gyro = Vector3D {
            x: row.take(8).unwrap(),
            y: row.take(9).unwrap(),
            z: row.take(10).unwrap(),
        };
        let pose = Orientation {
            roll: Angle::from_degrees(row.take(11).unwrap()),
            pitch: Angle::from_degrees(row.take(12).unwrap()),
            yaw: Angle::from_degrees(row.take(13).unwrap()),
            heading_accuracy: Angle::from_degrees(row.take(14).unwrap()),
        };
        let mag = Vector3D {
            x: row.take(15).unwrap(),
            y: row.take(16).unwrap(),
            z: row.take(17).unwrap(),
        };
        // let altitude: f64 = row.take(18).unwrap();
        // let temperature: Option<f64> = row.take(19);
        let altitude: Option<f64> = match from_value_opt(row[18].clone()) {
            Ok(v) => Some(v),
            Err(_e) => None,
        };
        let temperature: Option<f64> = match from_value_opt(row[19].clone()) {
            Ok(v) => Some(v),
            Err(_e) => None,
        };
        let cpu_temp: Option<f64> = match from_value_opt(row[20].clone()) {
            Ok(v) => Some(v),
            Err(_e) => None,
        };

        let uploaded: bool = row.take(21).unwrap();
        let confirmed: bool = row.take(22).unwrap();
        ImuRow {
            lineno,
            uuid,
            pitime,
            gps_time,
            sequence,
            accel,
            gyro,
            pose,
            mag,
            altitude,
            temperature,
            cpu_temp,
            uploaded,
            confirmed,
        }
    }
}

impl DatabaseName for Imu<'_> {
    fn name(&self) -> String {
        "imu".to_string()
    }
}

//  convertPressureToHeight() - the conversion uses the formula:
//
//  h = (T0 / L0) * ((p / P0)**(-(R* * L0) / (g0 * M)) - 1)
//
//  where:
//  h  = height above sea level
//  T0 = standard temperature at sea level = 288.15
//  L0 = standard temperature elapse rate = -0.0065
//  p  = measured pressure
//  P0 = static pressure = 1013.25 (but can be overridden)
//  g0 = gravitational acceleration = 9.80665
//  M  = mloecular mass of earth's air = 0.0289644
//  R* = universal gas constant = 8.31432
//
//  Given the constants, this works out to:
//
//  h = 44330.8 * (1 - (p / P0)**0.190263)

#[allow(dead_code)]
pub struct Imu<'a> {
    imu_type: ImuType,
    uuid: u64,
    pool: Pool,
    // conn: PooledConn,
    sensehat: Option<SenseHat<'a>>,
    twohat: Option<TwoHat>,
    altitude: f64,
    temperature: Option<Temperature>,
    temp_cpu: Option<Temperature>,
    i: u64,
    uploaded: bool,
    confirmed: bool,
    imu_tx: Option<mpsc::Sender<DeviceReporting>>,
    // gps_time: Arc<Mutex<Option<chrono::DateTime<Utc>>>>
    sequence: u32,
}
impl Imu<'_> {
    pub fn new(
        uuid: u64,
        imu_tx: Option<mpsc::Sender<DeviceReporting>>,
        gps_time: Arc<Mutex<Option<chrono::DateTime<Utc>>>>,
        pool: Pool,
    ) -> Imu<'static> {
        // pub fn new(uuid: u64, imu_tx: mpsc::Sender<DeviceReporting>, pool: Pool) -> Imu<'static> {
        info!("Starting sensehat IMU data logging...");

        // let conn = pool.get_conn().unwrap();

        let imu_type = which_imu();

        let (sensehat, twohat) = if imu_type.unwrap() == ImuType::SenseHatType {
            (
                Some(
                    SenseHat::new(&pool, uuid, gps_time).expect("Couldn't create Sense HAT object"),
                ),
                None,
            )
        } else if imu_type.unwrap() == ImuType::TwoHatType {
            (
                None,
                Some(TwoHat::new(&pool, uuid, gps_time).expect("Couldn't create Twohat object")),
            )
        } else {
            panic!("This doesn't have a TwoHat or a SenseHat")
        };

        thread::sleep(time::Duration::from_millis(3000));

        let conn = pool.get_conn().unwrap();
        let sequence = get_highest("imu", conn, uuid);
        trace!("IMU sequence starts today at {}", sequence);
        // trace!("GPS time in imu is {:?}", gps_time);

        Imu {
            imu_type: imu_type.unwrap(),
            uuid,
            pool,
            // conn,
            sensehat,
            twohat,
            altitude: 0.0, //pressure.as_hectopascals(),
            temperature: None,
            temp_cpu: None,
            i: 0,
            uploaded: false,
            confirmed: false,
            imu_tx,
            // gps_time,
            sequence,
        }
    }

    pub fn cycle(&mut self, n_iterations: u64, is_bridge: bool) {
        // println!("------ in IMU::cycle");
        match self.imu_type {
            ImuType::SenseHatType => self
                .sensehat
                .as_mut()
                .expect("Sensehat has disappeared since start of trucklog")
                .cycle(n_iterations, is_bridge),
            ImuType::TwoHatType => self
                .twohat
                .as_mut()
                .expect("Twohat has disappeared since start of trucklog")
                .cycle(n_iterations, is_bridge),
            // ImuType::NoImu | ImuType::Fake => panic!("Trying to cycle non-existing IMU"),
        }
    }

    /// Utility function
    #[allow(dead_code)]
    pub fn convert_pressure_to_height(pressure: f64, static_pressure: f64) -> f64 {
        44330.8 * (1.0 - (pressure / static_pressure).powf(0.190263))
    }

    pub fn format_imu(imu: &ImuData) -> (Vector3D, Vector3D, Vector3D, Orientation) {
        let z = Vector3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let zp = Orientation {
            roll: Angle::from_degrees(0.0),
            pitch: Angle::from_degrees(0.0),
            yaw: Angle::from_degrees(0.0),
            heading_accuracy: Angle::from_degrees(0.0),
        };

        let accel = match &imu.inertial {
            Err(_e) => z,
            Ok(inertial) => match inertial.accel {
                None => z,
                Some(a) => a,
            },
        };

        let gyro = match &imu.inertial {
            Err(_e) => z,
            Ok(inertial) => match inertial.gyro {
                None => z,
                Some(a) => a,
            },
        };

        let mag = match &imu.inertial {
            Err(_e) => z,
            Ok(inertial) => match inertial.mag {
                None => z,
                Some(a) => a,
            },
        };

        let pose = match &imu.inertial {
            Err(_e) => zp,
            Ok(inertial) => match inertial.pose {
                None => zp,
                Some(a) => a,
            },
        };
        (accel, gyro, mag, pose)
    }

    /// Write ImuData as returned by sensehat to the database
    /// imu is a Result, containing fields that are Options
    pub fn log_to_db(
        uuid: u64,
        sequence: u32,
        conn: &mut PooledConn,
        gps_time: Arc<Mutex<Option<chrono::DateTime<Utc>>>>,
        imu: &ImuData,
        altitude: f64,
        temperature: Option<Temperature>,
        temp_cpu: Option<Temperature>,
    ) {
        let (accel, gyro, mag, pose) = Self::format_imu(imu);

        let local_gps_time = *gps_time.lock().unwrap();

        let now = SystemTime::now();
        let dt_now: DateTime<Utc> = now.into();

        conn.exec_drop(
            "insert into imu (
                    uuid, 
                    pitime, gps_time, sequence,
                    x_accel, y_accel, z_accel,
                    x_gyro, y_gyro, z_gyro,
                    roll_pose, pitch_pose, yaw_pose,
                    heading_accuracy,
                    x_mag, y_mag, z_mag,
                    altitude, temperature,
                    temp_cpu,
                    uploaded, confirmed
                )
                values(:uuid, 
                    :pitime, :gps_time, :sequence,
                    :x_accel, :y_accel, :z_accel,
                    :x_gyro, :y_gyro, :z_gyro,
                    :roll_pose, :pitch_pose, :yaw_pose,
                    :heading_accuracy,
                    :x_mag, :y_mag, :z_mag,
                    :altitude, :temperature,
                    :temp_cpu,
                    :uploaded, :confirmed
                    )",
            params! {
                "uuid" => uuid,
                "pitime" => datetime_rust_to_sql(dt_now),
                "gps_time" => datetime_opt_to_sql(local_gps_time),
                "sequence" => sequence,
                "x_accel" => accel.x,
                "y_accel" => accel.y,
                "z_accel" => accel.z,
                "x_gyro" => gyro.x,
                "y_gyro" => gyro.y,
                "z_gyro" => gyro.z,
                "roll_pose" => pose.roll.as_degrees(),
                "pitch_pose" => pose.pitch.as_degrees(),
                "yaw_pose" => pose.yaw.as_degrees(),
                "heading_accuracy" => pose.heading_accuracy.as_degrees(),
                "x_mag" => mag.x,
                "y_mag" => mag.y,
                "z_mag" => mag.z,
                "altitude" => altitude,
                "temperature" => match temperature {
                    Some(v) => Some(v.as_celsius()),
                    None => None,
                },
                "temp_cpu" => match temp_cpu {
                    Some(v) => Some(v.as_celsius()),
                    None => None,
                },
                "uploaded" => false,
                "confirmed" => false,
            },
        )
        .expect("Failed to write to imu database in IMU:log_to_db");
    }

    /// Write a data summary to screen if required
    #[allow(dead_code)]
    fn log_to_screen(
        &self,
        imu: &ImuData,
        altitude: f64,
        temperature: f64,
        temp_cpu: f64,
    ) -> String {
        let (accel, gyro, mag, pose) = Self::format_imu(imu);

        let now = SystemTime::now();
        let dt_now: DateTime<Utc> = now.into();

        let roll: f64 = pose.roll.as_degrees();
        let pitch: f64 = pose.pitch.as_degrees();
        let yaw: f64 = pose.yaw.as_degrees();

        let heading: f64 = pose.heading_accuracy.as_degrees();

        format!(
            "IMU: {}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.1}, {:.1}, {:.2}, {:.2}, {:.2} {:.2}, {:.2}, {:.2}, {:.1?}, {:.1?}\r",
            dt_now,
            accel.x,
            accel.y,
            accel.z,
            gyro.x * 1000.0,
            gyro.y * 1000.0,
            gyro.z * 1000.0,
            mag.x,
            mag.y,
            mag.z,
            roll,
            pitch,
            yaw,
            heading,
            altitude,
            temperature,
            temp_cpu,
        )
    }
}
