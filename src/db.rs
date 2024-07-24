use rusqlite::{params, Connection, Result};
use imu::{ImuVec, Orientation, Vector3D, Inertial, ImuData} ;
use rand::Rng;

pub mod imu {
    tonic::include_proto!("imu");
}

fn main(){
    // println!("creating");
    // create_imu_table();

    // println!("filling");
    // fill_imu(100000);     //11s total time for 10000 rows, but was printing and debug
                                    //9.7s for 10000, release and no printing. 0.46 user, 2.29 system
                                    // For 1-million, 15:56.9 total, 42.1 user 238.2 system - slighly 
                                    // faster than scaled from 1000

    // read_imu_table();
}

pub fn insert_imu_line()->Result<(), rusqlite::Error>{
    let path = "/Users/drv201/Code/move_sql5/my_imu.db3";      // Errors with full path?
    let conn = Connection::open(path)?;//?;
     
    conn.execute("INSERT INTO imu (
        uuid,
        pitime, gps_time, sequence, 
        x_accel, y_accel, z_accel,
        x_gyro, y_gyro, z_gyro, 
        roll_pose, pitch_pose, yaw_pose, heading_accuracy, 
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
        1781003456, 1781003457, 0,  // pitime, gps_time, sequence
        in_range(-1000., 1000.), in_range(-1000., 1000.), in_range(-10000., 10000.),
        in_range(-500., 500.), in_range(-500., 500.), in_range(-100., 100.),
        in_range(-180., 180.), in_range(-180., 180.), in_range(-180., 180.),
        in_range(1., 20.),
        in_range(-18000., 18000.), in_range(-18000., 18000.), in_range(-18000., 18000.),
        in_range(91000., 106200.), in_range(-20.0, 50.0), in_range(20.0, 80.),
        0, 0],)?;
    
    let _ = conn.close();
    Ok(())
}


pub fn in_range(start: f32, stop: f32)->f64{

    let mut rng = rand::thread_rng();    
    let y: f32 = rng.gen();
    let range = stop-start;
    let value = start + y*range;
    value.into()
}


pub fn fill_imu(n_entries: usize){
    for _ in 0..n_entries{
        let _a = insert_imu_line();
    }
}

pub fn create_imu_table() -> Result<(), rusqlite::Error> {
    let path = "./my_imu.db3";      // Errors with full path?
    let conn = Connection::open(path)?;

    // lineno needs to be exactly INTEGER PRIMARY KEY to act as a ROWID
    let query = "
        CREATE TABLE imu 
        ( lineno INTEGER PRIMARY KEY NULL,
            uuid BIGINT NOT NULL, 
            pitime BINGINT NOT NULL, 
            gps_time BIGINT NOT NULL, 
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
        )";
        // Note: no comma after last line or it won't compile

    conn.execute(query, ())?; 
    let _ = conn.close();
    Ok(())
}

/// Read n records from the IMU table and return them as an ImuVec
pub fn read_imu_table(n : usize) -> Result<(), rusqlite::Error> {

    let records = ImuVec{
        data: Vec::new(),
    };

    let path = "./my_imu.db3";      // Errors with full path?
    let conn = Connection::open(path)?;

    let query = format!("SELECT * FROM imu WHERE uploaded = false ORDER BY lineno ASC LIMIT {};",n);
    let mut stmt = conn.prepare(&query)?;

    let imu_iter = stmt.query_map([], |row|{

        Ok( 
            // uuid : row.get(1)?,
            // gps_time : row.get(3)?,
            
            ImuData {

            sequence : row.get(4)?,
            timestamp : row.get(2)?,

            inertial: Some(Inertial {
                accel: Some(Vector3D {
                    x: row.get(5)?,
                    y: row.get(6)?,
                    z: row.get(7)?,
                }),
                gyro: Some(Vector3D {
                    x: row.get(8)?,
                    y: row.get(9)?,
                    z: row.get(10)?,
                }),
                pose: Some(Orientation {
                    roll: row.get(11)?,
                    pitch: row.get(12)?,
                    yaw: row.get(13)?,
                    heading_accuracy:row.get(14)?,
                }),
                mag: Some(Vector3D{
                    x: row.get(15)?,
                    y: row.get(16)?,
                    z: row.get(17)?,                    
                }),
            }),
            pressure: row.get(18)?, 
            temperature: row.get(19)?, 
            temp_cpu: row.get(20)?, 
     
            // uploaded: row.get(21)?,  
            // confirmed: row.get(22)?,
        })
    })?;



    let results = conn.execute(&query, ())?; 


    Ok(())
}