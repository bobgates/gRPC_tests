use measurements::Pressure;
use rusqlite::{ params, Connection, Result}; //ffi::SQLITE_NULL,
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
mod data_defs;
use crate::data_defs::{generate_imu_data, ImuData, Inertial, Orientation, Vector3D};
use sqlite_tests::ThreadPool;

pub fn make_imu(conn: &mut Connection)->Result<()>{
    let _ = conn.execute("DROP TABLE IF EXISTS imu",());

    conn.execute("CREATE TABLE imu 
                    ( lineno INTEGER PRIMARY KEY, 
                        uuid BIGINT NOT NULL,
                        pitime DATETIME(6) NOT NULL,
                        gps_time DATETIME(3), 
                        sequence INT NOT NULL, 
                        x_accel FLOAT NOT NULL,  
                        y_accel FLOAT NOT NULL,  
                        z_accel FLOAT NOT NULL,  
                        x_gyro FLOAT NOT NULL,  
                        y_gyro FLOAT NOT NULL,  
                        z_gyro FLOAT NOT NULL,
                        roll_pose FLOAT NOT NULL,  
                        pitch_pose FLOAT NOT NULL,  
                        yaw_pose FLOAT NOT NULL, 
                        heading_accuracy FLOAT NOT NULL,
                        x_mag FLOAT NOT NULL, 
                        y_mag FLOAT NOT NULL,  
                        z_mag FLOAT NOT NULL,  
                        altitude FLOAT, 
                        temperature FLOAT, 
                        temp_cpu FLOAT, 
                        uploaded int NOT NULL,  
                        confirmed int NOT NULL
                    )",
                    (),
                )?;
                Ok(())
}

pub fn make_test(conn: &mut Connection)->Result<()>{
    let _ = conn.execute("DROP TABLE IF EXISTS test",());

    conn.execute("CREATE TABLE test
                    ( lineno INTEGER PRIMARY KEY, 
                        uuid BIGINT NOT NULL,
                        pitime DATETIME(6) NOT NULL,
                        uploaded int NOT NULL,  
                        confirmed int NOT NULL
                    )",
                    (),
                )?;
                Ok(())
}

pub fn fill_test(conn: &mut Connection, n_rows: usize)->Result<()>{
    for i in 0..n_rows {
        println!("{} ",i);
        conn.execute("INSERT INTO test (
            uuid,
            pitime, 
            uploaded, confirmed)
            VALUES (
                ?1,
                ?2, 
                ?3, 
                ?4)",
         params![ 
            0x12367ABC,
            1781003456,
            0, 0],)?;
    }
    Ok(())
}

pub fn fill_imu(conn: &mut Connection)->Result<()>{

    let imu = generate_imu_data();

    let inertial = imu.inertial.unwrap();
    let accel = inertial.accel.unwrap();
    let pose = inertial.pose.unwrap();
    let gyro = inertial.gyro.unwrap();
    let mag = inertial.mag.unwrap();

    // for i in 0..n_rows {
        print!(".");
        conn.execute("INSERT INTO imu (
            uuid,
            pitime, gps_time, sequence, 
            x_accel, y_accel, z_accel,
            x_gyro, y_gyro, z_gyro, 
            roll_pose, pitch_pose, yaw_pose, 
            heading_accuracy, 
            x_mag, y_mag, z_mag, 
            altitude, temperature, temp_cpu, 
            uploaded, confirmed)
            VALUES (
                ?1,
                ?2, ?3, ?4,
                ?5, ?6, ?7, 
                ?8, ?9, ?10, 
                ?11, ?12, ?13, 
                ?14,
                ?15, ?16, ?17, 
                ?18, ?19, ?20, 
                ?21, ?22)",
         params![
            0x12367ABCABAB as i64,          //sqlite handles this i64 just fine
            1781003456, 1781003457, 0,
            accel.x, accel.y, accel.z,
            gyro.x, gyro.y, gyro.z,
            pose.roll, pose.pitch, pose.yaw,
            pose.heading_accuracy,
            mag.x, mag.y, mag.z,
            imu.pressure.unwrap().as_pascals(), imu.temperature.unwrap(), imu.temp_cpu.unwrap(),
            0, 0],)?;
    // };
    Ok(())
} 

#[derive(Debug)]

pub struct ImuShort{
    line: usize,
    uuid: u64,
    pitime: u64,
    gps_time: u64, 
    sequence: usize,

    acccel_x: f32, 
    accel_y: f32, 
    accel_z: f32,

    gyro_x: f32, 
    gyro_y: f32, 
    gyro_z: f32, 

    pose_roll: f32, 
    pose_pitch: f32, 
    pose_yaw: f32, 

    pose_heading_accuracy: f32, 

    mag_x: f32, 
    mag_y: f32, 
    mag_z: f32, 

    altitude: f32, 
    temperature: f32, 
    temp_cpu: f32, 
    uploaded: usize,   
    confirmed: usize,
}

pub fn convert_imu_short_to_data(short: ImuShort)-> ImuData {

    ImuData{
        timestamp: short.pitime,
        inertial: Ok( Inertial {
            pose: Some( Orientation{
                roll: short.pose_roll,
                pitch: short.pose_pitch,
                yaw: short.pose_yaw,
                heading_accuracy: short.pose_heading_accuracy,
            }),     
            gyro: Some( Vector3D{
                x: short.gyro_x,
                y: short.gyro_y,
                z: short.gyro_z,
            }),
            accel: Some( Vector3D{
                x: short.acccel_x,
                y: short.accel_y,
                z: short.accel_z,
            }),
            mag: Some( Vector3D{
                x: short.mag_x,
                y: short.mag_y,
                z: short.mag_z,
            }),
        }),
        pressure: Some(Pressure::from_pascals(short.altitude.into())),
        temperature: Some(short.temperature),
        temp_cpu: Some(short.temp_cpu),

    }

}


pub fn get_earliest_n(conn: &mut Connection, n: usize)->Result<()>{//->ImuData{

    let mut stmt = conn.prepare(&format!("SELECT * FROM imu WHERE uploaded = false ORDER BY lineno ASC LIMIT {}", n))?;

    let imu_iter = stmt.query_map([], |row| {
        Ok(ImuShort{ line: row.get(0)?,
            uuid: row.get(1)?,
            pitime: row.get(2)?,
            gps_time: row.get(3)?,
            sequence: row.get(4)?,
        
            acccel_x: row.get(5)?,
            accel_y: row.get(6)?,
            accel_z: row.get(7)?,
        
            gyro_x: row.get(8)?,
            gyro_y: row.get(9)?,
            gyro_z: row.get(10)?,
        
            pose_roll: row.get(11)?,
            pose_pitch: row.get(12)?,
            pose_yaw: row.get(13)?,
        
            pose_heading_accuracy: row.get(14)?,
        
            mag_x: row.get(15)?,
            mag_y: row.get(16)?,
            mag_z: row.get(17)?,
        
            altitude: row.get(18)?,
            temperature: row.get(19)?,
            temp_cpu: row.get(20)?,
            uploaded: row.get(21)?,
            confirmed: row.get(22)?,
        })
    })?;

    for imu in imu_iter {
        println!("{:?}",imu)
    }

    Ok(())
}





fn main()->Result<(), rusqlite::Error> {
    // let mut conn = Connection::open("/Users/drv201/Code/sqlite_tests/test.db").unwrap();

    //let _ = make_imu(&mut conn);     // Uncomment if you need to make the table
    // let _ = make_test(&mut conn);    // Uncomment if you need to make the table

    // for i in 0..100 {
    //     let _ = fill_imu(&mut conn);
    // }
    // println!("");

    // let _ = get_earliest_n(&mut conn,10);


    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
        // println!("Connection established!");
        // handle_connection(stream);
    }


    Ok(())
}


pub fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // println!("Reqest: {http_request:#?}");

    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("/Users/drv201/Code/sqlite_tests/src/hello.html").unwrap();
    let length = contents.len();

    let response = 
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();

}


    
