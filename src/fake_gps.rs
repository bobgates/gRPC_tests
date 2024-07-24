use std::time::SystemTime;
use crate::gps;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]

use gps::{GpsVec, GpsData};



/// Encode five database fields into one u32
pub fn encode_fields(status: u8, nsats: u8, valid: bool, uploaded: bool, confirmed: bool)->u32{

    (status as u32 * 65536 + 
    nsats as u32 * 256 + 
    if valid {4} else {0} + 
    if uploaded {2} else {0} + 
    if confirmed {1} else {0}).into()

}

/// Decode the u32 back into five database fields
pub fn decode_fields(num: u32) -> (u8, u8, bool, bool, bool){
    
    let bytes = num.to_be_bytes();

    let confirmed = (num % 2) == 1;
    let valid = (num/2 %2) == 1;
    let uploaded = (num/4 %2) ==1;
    let nsats = bytes[2]; //(num/256 %256).into();
    let status = bytes[1]; //(num/65536 %256).into();
    
    (status, nsats, uploaded, valid, confirmed)
}

pub fn generate_gps_data(n : usize)->GpsVec{
    let mut d: Vec::<GpsData> = Vec::new();

    for i in 0..n as u64 {
        d.push(generate_gps_line(i.into()));
    }

    GpsVec{ data: d}
}

pub fn generate_gps_line(sequence : u32)->GpsData{

    let timestamp = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .expect("Unable to calculate current time in imu::cycle")
    .as_millis() as u64;
    
    let uuid = 0x1234567890AB;
    let pitime = 2000_000_000;
    let gps_time = 1999_999_999;
    let lat : f32 = 50.123456;
    let lon : f32 = -4.9987654;
    let alt : f32 = 100.45;
    let speed: f32 = 10.0;
    let track: f32 = 359.995566;
    let hdop : f32 = 12.4321;
    let status_nsats_vuc = encode_fields(1, 12, true, false, false);
    
    GpsData { 
        uuid,
        pitime,
        gps_time,
        sequence,
        lat,
        lon,
        alt,
        speed,
        track,
        status_nsats_vuc,
        hdop,
    }
}