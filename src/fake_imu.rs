use std::time::SystemTime;
use crate::imu;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]

use imu::{ImuVec, ImuData, Orientation, Inertial, Vector3D};

pub fn generate_imu_line(seq : u32)->ImuData{

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

    let inertial = Some(Inertial {
        pose: Some(orientation),
        gyro: Some(Vector3D { x: 0.01, y: 0.03, z: 18.5 }),
        accel: Some(Vector3D { x: 0.01, y: 0.03, z: 1.005 }),
        mag: Some(Vector3D { x: 28.3, y: 16.9, z: 11.2 }),
    });

    ImuData { 
        timestamp,
        inertial, 
        sequence: seq,
        pressure: 101320.0, 
        temperature: 23.0, 
        temp_cpu: 77.3,
    }
}

pub fn generate_imu_data(n : usize)->ImuVec{
    let mut d: Vec::<ImuData> = Vec::new();

    for i in 0..n as u32 {
        d.push(generate_imu_line(i.into()));
    }

    ImuVec{ data: d}
}




