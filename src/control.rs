use serde::Deserialize;
use crate::{checksum, constants};

/// Trait for camera commands
pub trait Command {
    fn to_bytes(&self) -> Vec<u8>;
}

/// Trait for HTTP API queries
pub trait HTTPQuery {
    fn to_string(&self) -> String;
}

/// Enums for hardcoded simple commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A8MiniSimpleCommand {
    AutoCenter = 0, // handled ACK (sta)
    RotateUp = 1, // handled ACK (sta)
    RotateDown = 2, // handled ACK (sta)
    RotateRight = 3, // handled ACK (sta)
    RotateLeft = 4, // handled ACK (sta)
    StopRotation = 5, // handled ACK (sta)
    ZoomIn = 6, // handled ACK (sta)
    ZoomOut = 7, // handled ACK (sta)
    ZoomMax = 8,
    MaxZoomInformation = 9,
    FocusIn = 10,
    FocusOut = 11,
    TakePicture = 12, // no ACK
    RecordVideo = 13, // no ACK
    Rotate100100 = 14,
    CameraInformation = 15,
    AutoFocus = 16, // handled ACK (sta)
    HardwareIDInformation = 17,
    FirmwareVersionInformation = 18,
    SetLockMode = 19,
    SetFollowMode = 20,
    SetFPVMode = 21,
    AttitudeInformation = 22,
    SetVideoOutputHDMI = 23,
    SetVideoOutputCVBS = 24,
    SetVideoOutputOff = 25,
    LaserRangefinderInformation = 26,
    RebootCamera = 27,
    RebootGimbal = 28,
}

impl Command for A8MiniSimpleCommand {
    fn to_bytes(&self) -> Vec<u8> {
        constants::HARDCODED_COMMANDS[*self as usize].to_vec()
    }
}


/// Enums for commands that require continuous values for data field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A8MiniComplexCommand {
    SetYawPitchSpeed(i8, i8),
    SetYawPitchAngle(i16, i16),
}

impl Command for A8MiniComplexCommand {
    fn to_bytes(&self) -> Vec<u8> {
        match *self {
            A8MiniComplexCommand::SetYawPitchSpeed(v_yaw, v_pitch) => {
                let mut byte_arr: Vec<u8> = vec![0x55,0x66,0x01,0x02,0x00,0x00,0x00,0x07];

                byte_arr.push(v_yaw.clamp(-100, 100) as u8);
                byte_arr.push(v_pitch.clamp(-100, 100) as u8);

                byte_arr.extend_from_slice(&checksum::crc16_calc(&byte_arr, 0));

                byte_arr
            },
            A8MiniComplexCommand::SetYawPitchAngle(theta_yaw, theta_pitch) => {
                let mut byte_arr: Vec<u8> = vec![0x55,0x66,0x01,0x04,0x00,0x00,0x00,0x0e];

                byte_arr.extend_from_slice(&theta_yaw.clamp(-1350, 1350).to_be_bytes());
                byte_arr.extend_from_slice(&theta_pitch.clamp(-900, 250).to_be_bytes());

                byte_arr.extend_from_slice(&checksum::crc16_calc(&byte_arr, 0));

                byte_arr
            },
        }
    }
}


/// Enums for simple HTTP queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A8MiniSimpleHTTPQuery {
    GetDirectoriesPhotos,
    GetDirectoriesVideos,
    GetMediaCountPhotos,
    GetMediaCountVideos,
}

impl HTTPQuery for A8MiniSimpleHTTPQuery {
    fn to_string(&self) -> String {
        match *self {
            A8MiniSimpleHTTPQuery::GetDirectoriesPhotos => "http://192.168.144.25:82/cgi-bin/media.cgi/api/v1/getdirectories?media_type=0".to_string(),
            A8MiniSimpleHTTPQuery::GetDirectoriesVideos => "http://192.168.144.25:82/cgi-bin/media.cgi/api/v1/getdirectories?media_type=1".to_string(),
            A8MiniSimpleHTTPQuery::GetMediaCountPhotos => "http://192.168.144.25:82/cgi-bin/media.cgi/api/v1/getmediacount?media_type=0&path=101SIYI_IMG".to_string(),
            A8MiniSimpleHTTPQuery::GetMediaCountVideos => "http://192.168.144.25:82/cgi-bin/media.cgi/api/v1/getmediacount?media_type=1&path=101SIYI_VID".to_string(),
        }
    }
}


/// Enums for complex HTTP queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A8MiniComplexHTTPQUery {
    GetPhoto(u8),
    GetVideo(u8),
}

impl HTTPQuery for A8MiniComplexHTTPQUery {
    fn to_string(&self) -> String {
        match *self {
            A8MiniComplexHTTPQUery::GetPhoto(photo_ind) => format!("http://192.168.144.25:82/photo/101SIYI_IMG/IMG_{:0>4}.jpg", photo_ind),
            A8MiniComplexHTTPQUery::GetVideo(video_ind) => format!("http://192.168.144.25:82/video/101SIYI_VID/VID_{:0>4}.mpeg", video_ind),
        }
    }
}


/// Camera attitude information
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct A8MiniAtittude {
    pub theta_yaw: i16,
    pub theta_pitch: i16,
    pub theta_roll: i16,
    pub v_yaw: i16,
    pub v_pitch: i16,
    pub v_roll: i16,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_command_creation_angle() {
        let computed_command = A8MiniComplexCommand::SetYawPitchAngle(130, -20).to_bytes();
        let expected_command: [u8; 14] = [0x55,0x66,0x01,0x04,0x00,0x00,0x00,0x0e,0x00,0x82,0xff,0xec,0x8f,0xad];
        assert_eq!(computed_command, expected_command);
    }

    #[test]
    fn test_complex_command_creation_speed() {
        let computed_command = A8MiniComplexCommand::SetYawPitchSpeed(104, -20).to_bytes();
        let expected_command: [u8; 12] = [0x55,0x66,0x01,0x02,0x00,0x00,0x00,0x07,0x64,0xec,0xbd,0xdf];
        assert_eq!(computed_command, expected_command);
    }

    #[test]
    fn test_byte_deserialization() {
        let attitude_bytes: &[u8] = &[0x28,0x00,0x32,0x00,0x3c,0x00,0x04,0x00,0x05,0x00,0x06,0x00];

        // Note: little endian deserialize
        let computed_attitude_info: A8MiniAtittude = bincode::deserialize(attitude_bytes).unwrap();
        
        let expected_attitude_info = A8MiniAtittude {
            theta_yaw: 40,
            theta_pitch: 50,
            theta_roll: 60,
            v_yaw: 4,
            v_pitch: 5,
            v_roll: 6,
        };

        assert_eq!(computed_attitude_info, expected_attitude_info);
    }
}

