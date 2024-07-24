// use imu::data_server_client::DataServerClient;
mod fake_imu;
use crate::fake_imu::generate_imu_data;
mod fake_gps;
use crate::fake_gps::generate_gps_data;

use imu::imu_data_server_client::ImuDataServerClient;
use gps::gps_data_server_client::GpsDataServerClient;

pub mod gps {
    tonic::include_proto!("gps");
}
pub mod imu {
    tonic::include_proto!("imu");
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut imu_client = ImuDataServerClient::connect("http://[::1]:50051").await?;
    let mut gps_client = GpsDataServerClient::connect("http://[::1]:50051").await?;

    let no_of_lines = 940;

    let request = tonic::Request::new(generate_imu_data(no_of_lines));
    let response = imu_client.send_imu(request).await?;

    println!("IMU RESPONSE={:?}", response);

    let request = tonic::Request::new(generate_gps_data(no_of_lines));
    let response = gps_client.send_gps(request).await?;

    println!("GPS RESPONSE={:?}", response);

    let request = tonic::Request::new(generate_imu_data(no_of_lines));
    let response = imu_client.send_imu(request).await?;

    println!("IMU RESPONSE={:?}", response);



    Ok(())
}