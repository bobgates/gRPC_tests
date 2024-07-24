use measurements::Pressure;
// use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]

#[derive(Debug)]
pub struct Orientation {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub heading_accuracy: f32,
}

#[derive(Debug)]
pub struct Vector3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug)]
pub struct Inertial {
    pub pose: Option<Orientation>,
    pub gyro: Option<Vector3D>,
    pub accel: Option<Vector3D>,
    pub mag: Option<Vector3D>,
}


#[derive(Debug)]
pub enum ImuError {
    NotReady,
}


#[derive(Debug)]
pub struct ImuData {
    pub timestamp: u64,
    pub inertial: Result<Inertial, ImuError>,
    pub pressure: Option<Pressure>,
    pub temperature: Option<f32>,
    pub temp_cpu: Option<f32>,
    // pub humidity: Option<f64>,
}

pub fn generate_imu_data()->ImuData{

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Unable to calculate current time in imu::cycle")
        .as_millis() as u64;
    
    let orientation = Orientation{
        roll: 10.4,
        pitch: 0.0,
        yaw:188.9,
        heading_accuracy: 3.2,
    };

    let inertial = Ok(Inertial {
        pose: Some(orientation),
        gyro: Some(Vector3D { x: 0.01, y: 0.03, z: 18.5 }),
        accel: Some(Vector3D { x: 0.01, y: 0.03, z: 1.005 }),
        mag: Some(Vector3D { x: 28.3, y: 16.9, z: 11.2 }),
    });

    ImuData { 
        timestamp,
        inertial, 
        pressure: Some(Pressure::from_pascals(101320.0)), 
        temperature: Some(23.0), 
        temp_cpu: Some(77.3),
    }

}