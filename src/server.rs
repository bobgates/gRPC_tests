use std::thread;
use tonic::{transport::Server, Request, Response, Status};

use imu::imu_data_server_server::{ImuDataServer, ImuDataServerServer}     ;
use gps::gps_data_server_server::{GpsDataServer, GpsDataServerServer}     ;
use imu::{ImuVec, ImuReply};
use gps::{GpsVec, GpsReply};


pub mod imu {
    tonic::include_proto!("imu");
}
pub mod gps {
    tonic::include_proto!("gps");
}


#[derive(Debug, Default)]
pub struct ImuDataSource {}

#[tonic::async_trait]
impl ImuDataServer for ImuDataSource {
    async fn send_imu(
        &self,
        request: Request<ImuVec>,
    ) -> Result<Response<ImuReply>, Status> {
        // println!("Got a request: {:?}", request);

        let n_lines = request.into_inner().data.len();

        let reply = ImuReply {
            message: format!("{} IMU lines received!", n_lines), //, request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

#[derive(Debug, Default)]
pub struct GpsDataSource {}

#[tonic::async_trait]
impl GpsDataServer for GpsDataSource {
    async fn send_gps(
        &self,
        request: Request<GpsVec>,
    ) -> Result<Response<GpsReply>, Status> {

        let n_lines = request.into_inner().data.len();

        let reply = GpsReply {
            message: format!("{} GPS lines received!", n_lines), 
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = ImuDataSource::default();
    let gps_data = GpsDataSource::default();


    Server::builder()
        .add_service(ImuDataServerServer::new(greeter))
        .add_service(GpsDataServerServer::new(gps_data))
        .serve(addr)
        .await?;

    Ok(())

}
