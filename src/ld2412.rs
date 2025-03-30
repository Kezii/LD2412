use crate::{RadarDriver, RadarLLFrame};
use log::error;
use smallvec::SmallVec;

#[derive(Debug, Clone, Copy)]
pub enum RadarResolution {
    Cm75 = 0x00,
    Cm50 = 0x01,
    Cm25 = 0x02,
}

#[derive(Debug)]
pub enum Ld2412Command {
    /// send this command to enable configuration mode, otherwise the radar will ignore all other commands
    EnableConfiguration,
    /// send this command to end configuration mode, so the radar will start to work
    EndConfiguration,
    Resolution(RadarResolution),
    ReadResolution,
    /// min_distance, max_distance, unoccupied_duration, polarity
    BasicParameters(u8, u8, u16, bool),
    ReadBasicParameters,
    EngineeringModeOn,
    EngineeringModeOff,
    MotionSensitivity([u8; 14]),
    ReadMotionSensitivity,
    StaticSensitivity([u8; 14]),
    ReadStaticSensitivity,
    EnterBackgroundCorrection,
    ReadBackgroundCorrection,
    FirmwareVersion,
    BaudRate(u32),
    FactoryReset,
    Reboot,
    BluetoothOn,
    BluetoothOff,
    MacAddress,
    /// mode, threshold
    LightsensorMode(u8, u8),
    ReadLightsensorMode,
}

impl RadarDriver for Ld2412Command {
    fn get_opcode(&self) -> u16 {
        match self {
            Ld2412Command::EnableConfiguration => 0x00FF,
            Ld2412Command::EndConfiguration => 0x00FE,
            Ld2412Command::Resolution(_) => 0x0001,
            Ld2412Command::ReadResolution => 0x0011,
            Ld2412Command::BasicParameters(_, _, _, _) => 0x0002,
            Ld2412Command::ReadBasicParameters => 0x0012,
            Ld2412Command::MotionSensitivity(_) => 0x0003,
            Ld2412Command::ReadMotionSensitivity => 0x0013,
            Ld2412Command::StaticSensitivity(_) => 0x0004,
            Ld2412Command::ReadStaticSensitivity => 0x0014,
            Ld2412Command::EnterBackgroundCorrection => 0x000B,
            Ld2412Command::ReadBackgroundCorrection => 0x001B,
            Ld2412Command::EngineeringModeOn => 0x0062,
            Ld2412Command::EngineeringModeOff => 0x0063,
            Ld2412Command::FirmwareVersion => 0x00A0,
            Ld2412Command::BaudRate(_) => 0x00A1,
            Ld2412Command::FactoryReset => 0x00A2,
            Ld2412Command::Reboot => 0x00A3,
            Ld2412Command::BluetoothOn => 0x00A4,
            Ld2412Command::BluetoothOff => 0x00A5,
            Ld2412Command::MacAddress => 0x00A6,
            Ld2412Command::LightsensorMode(_, _) => 0x000C,
            Ld2412Command::ReadLightsensorMode => 0x001C,
        }
    }

    fn serialize_data(&self, data: &mut SmallVec<[u8; 16]>) {
        match self {
            Ld2412Command::EnableConfiguration => {
                data.extend_from_slice(&[0x01, 0x00]);
            }
            Ld2412Command::EndConfiguration => {}
            Ld2412Command::Resolution(resolution) => {
                data.extend_from_slice(&[*resolution as u8, 0x00, 0x00, 0x00, 0x00, 0x00]);
            }
            Ld2412Command::ReadResolution => {}
            Ld2412Command::BasicParameters(
                min_distance,
                max_distance,
                unoccupied_duration,
                polarity,
            ) => {
                data.extend_from_slice(&[
                    *min_distance,
                    *max_distance,
                    (*unoccupied_duration & 0xFF) as u8,
                    ((*unoccupied_duration >> 8) & 0xFF) as u8,
                    if *polarity { 0x01 } else { 0x00 },
                    0x00,
                ]);
            }
            Ld2412Command::ReadBasicParameters => {}
            Ld2412Command::EngineeringModeOn => {}
            Ld2412Command::EngineeringModeOff => {}
            Ld2412Command::MotionSensitivity(sensitivity) => {
                data.extend_from_slice(sensitivity);
            }
            Ld2412Command::ReadMotionSensitivity => {}
            Ld2412Command::StaticSensitivity(sensitivity) => {
                data.extend_from_slice(sensitivity);
            }
            Ld2412Command::ReadStaticSensitivity => {}
            Ld2412Command::EnterBackgroundCorrection => {}
            Ld2412Command::ReadBackgroundCorrection => {}
            Ld2412Command::FirmwareVersion => {}
            Ld2412Command::BaudRate(baud_rate) => {
                let br: u16 = match baud_rate {
                    9600 => 0x0001,
                    19200 => 0x0002,
                    38400 => 0x0003,
                    57600 => 0x0004,
                    115200 => 0x0005,
                    230400 => 0x0006,
                    256600 => 0x0007,
                    460800 => 0x0008,
                    _ => panic!("Unknown baud rate"),
                };

                data.extend_from_slice(&[br as u8, (br >> 8) as u8]);
            }
            Ld2412Command::FactoryReset => {}
            Ld2412Command::Reboot => {}
            Ld2412Command::BluetoothOn => {
                data.extend_from_slice(&[0x01, 0x00]);
            }
            Ld2412Command::BluetoothOff => {
                data.extend_from_slice(&[0x00, 0x00]);
            }
            Ld2412Command::MacAddress => {
                data.extend_from_slice(&[0x01, 0x00]);
            }
            Ld2412Command::LightsensorMode(mode, threshold) => {
                data.extend_from_slice(&[*mode, *threshold]);
            }
            Ld2412Command::ReadLightsensorMode => {}
        }
    }
}

impl Ld2412Command {
    pub fn to_llframe(&self) -> RadarLLFrame {
        let mut data = SmallVec::new();
        self.serialize_data(&mut data);
        RadarLLFrame::CommandAckFrame(self.get_opcode(), data)
    }
}

// deserialization

#[derive(Debug)]
pub enum TargetState {
    Untargeted = 0x00,
    Campaign = 0x01,
    Stationary = 0x02,
    MotionStationary = 0x03,
    BottomNoiseDetectionInProgress = 0x04,
    BottomNoiseDetectionSuccessful = 0x05,
    BottomNoiseDetectionFailed = 0x06,
}

impl From<u8> for TargetState {
    fn from(item: u8) -> Self {
        match item {
            0x00 => TargetState::Untargeted,
            0x01 => TargetState::Campaign,
            0x02 => TargetState::Stationary,
            0x03 => TargetState::MotionStationary,
            0x04 => TargetState::BottomNoiseDetectionInProgress,
            0x05 => TargetState::BottomNoiseDetectionSuccessful,
            0x06 => TargetState::BottomNoiseDetectionFailed,
            _ => panic!("Unknown target state"),
        }
    }
}

#[derive(Debug)]
pub struct Ld2412TargetData {
    pub basic_target_data: BasicTargetData,
    pub engineering_mode_data: Option<EngineeringModeData>,
}

#[derive(Debug)]
pub struct BasicTargetData {
    pub state: TargetState,
    pub moving_target: Target,
    pub stationary_target: Target,
}

#[derive(Debug)]
pub struct EngineeringModeData {
    pub b1: u8,
    pub b2: u8,
    pub moving_gates: [u8; 14],
    pub stationary_gates: [u8; 14],
    pub light: u8,
}

#[derive(Debug)]
pub struct Target {
    distance: u16, // cm
    energy: u8,    // dB ??
}

fn read_basic_target_data(buffer: &[u8]) -> BasicTargetData {
    let moving_target = Target {
        distance: u16::from_le_bytes([buffer[1], buffer[2]]),
        energy: buffer[3],
    };

    let stationary_target = Target {
        distance: u16::from_le_bytes([buffer[4], buffer[5]]),
        energy: buffer[6],
    };

    BasicTargetData {
        state: buffer[0].into(),
        moving_target,
        stationary_target,
    }
}

impl Ld2412TargetData {
    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        match buffer {
            [datatype, 0xaa, targetdata @ .., 0x55, calibration] => {
                let target_data = match *datatype {
                    0x01 => {
                        let basic_target_data = read_basic_target_data(targetdata);

                        let light = targetdata[37];
                        let eng_data = EngineeringModeData {
                            b1: targetdata[7],
                            b2: targetdata[8],
                            moving_gates: targetdata[9..23].try_into().unwrap(),
                            stationary_gates: targetdata[23..37].try_into().unwrap(),

                            light,
                        };

                        Ld2412TargetData {
                            basic_target_data,
                            engineering_mode_data: Some(eng_data),
                        }
                    }
                    0x02 => {
                        let basic_target_data = read_basic_target_data(targetdata);

                        Ld2412TargetData {
                            basic_target_data,
                            engineering_mode_data: None,
                        }
                    }
                    _ => {
                        error!("Unknown datatype");
                        return None;
                    }
                };

                let _speed = (*calibration) as i8; //?

                Some(target_data)
            }
            _ => {
                error!("Intraframe is incorrect");
                None
            }
        }
    }
}
