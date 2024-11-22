use log::{error, info, trace};

#[derive(Debug)]
pub enum CommandsOpcode {
    EnableConfiguration = 0x00FF,
    EndConfiguration = 0x00FE,
    Resolution = 0x0001,
    ReadResolution = 0x0011,
    BasicParameters = 0x0002,
    ReadBasicParameters = 0x0012,
    MotionSensitivity = 0x0003,
    ReadMotionSensitivity = 0x0013,
    StaticSensitivity = 0x0004,
    ReadStaticSensitivity = 0x0014,
    EnterBackgroundCorrection = 0x000B,
    ReadBackgroundCorrection = 0x001B,
    EngineeringModeOn = 0x0062,
    EngineeringModeOff = 0x0063,
    FirmwareVersion = 0x00A0,
    BaudRate = 0x00A1,
    FactoryReset = 0x00A2,
    Reboot = 0x00A3,
    Bluetooth = 0x00A4,
    MacAddress = 0x00A5,
    LightsensorMode = 0x000C,
    ReadLightsensorMode = 0x001C,
}

pub fn command_serialize(command: CommandsOpcode, frame: &[u8]) -> Vec<u8> {
    trace!("Serializing command: {command:?} {:x?}", frame);
    let mut buffer = Vec::new();

    buffer.extend_from_slice(&[0xFD, 0xFC, 0xFB, 0xFA]);
    buffer.extend_from_slice(&(frame.len() as u16 + 2).to_le_bytes());
    buffer.extend_from_slice(&(command as u16).to_le_bytes());
    buffer.extend_from_slice(frame);
    buffer.extend_from_slice(&[0x04, 0x03, 0x02, 0x01]);

    buffer
}

pub fn enable_configuration() -> Vec<u8> {
    info!("Enabling configuration");
    command_serialize(CommandsOpcode::EnableConfiguration, &[0x01, 0x00])
}

pub fn end_configuration() -> Vec<u8> {
    info!("Ending configuration");
    command_serialize(CommandsOpcode::EndConfiguration, &[])
}

pub enum RadarResolution {
    cm75 = 0x00,
    cm50 = 0x01,
    cm25 = 0x02,
}

pub fn set_resolution(resolution: RadarResolution) -> Vec<u8> {
    let resolution = resolution as u8;
    info!("Setting resolution to {:?}", resolution);
    command_serialize(
        CommandsOpcode::Resolution,
        &[resolution, 0x00, 0x00, 0x00, 0x00, 0x00],
    )
}

pub fn read_resolution() -> Vec<u8> {
    info!("Reading resolution");
    command_serialize(CommandsOpcode::ReadResolution, &[])
}

pub fn set_basic_parameters(
    min_distance: u8,
    max_distance: u8,
    unoccupied_duration: u16,
    polarity: bool,
) -> Vec<u8> {
    info!("Setting basic parameters");
    command_serialize(
        CommandsOpcode::BasicParameters,
        &[
            min_distance,
            max_distance,
            unoccupied_duration as u8,
            (unoccupied_duration >> 8) as u8,
            if polarity { 0x01 } else { 0x00 },
            0x00,
        ],
    )
}

pub fn read_basic_parameters() -> Vec<u8> {
    info!("Reading basic parameters");
    command_serialize(CommandsOpcode::ReadBasicParameters, &[])
}

pub fn set_enable_engineering_mode() -> Vec<u8> {
    info!("Setting enable engineering mode");
    command_serialize(CommandsOpcode::EngineeringModeOn, &[])
}

pub fn set_disable_engineering_mode() -> Vec<u8> {
    info!("Setting disable engineering mode");
    command_serialize(CommandsOpcode::EngineeringModeOff, &[])
}

pub fn set_motion_sensitivity(sensitivity: [u8; 14]) -> Vec<u8> {
    info!("Setting motion sensitivity");
    command_serialize(CommandsOpcode::MotionSensitivity, &sensitivity)
}

pub fn read_motion_sensitivity() -> Vec<u8> {
    info!("Reading motion sensitivity");
    command_serialize(CommandsOpcode::ReadMotionSensitivity, &[])
}

pub fn set_static_sensitivity(sensitivity: [u8; 14]) -> Vec<u8> {
    info!("Setting static sensitivity");
    command_serialize(CommandsOpcode::StaticSensitivity, &sensitivity)
}

pub fn read_static_sensitivity() -> Vec<u8> {
    info!("Reading static sensitivity");
    command_serialize(CommandsOpcode::ReadStaticSensitivity, &[])
}

pub fn set_enter_background_correction() -> Vec<u8> {
    info!("Setting enter background correction");
    command_serialize(CommandsOpcode::EnterBackgroundCorrection, &[])
}

pub fn read_background_correction() -> Vec<u8> {
    info!("Reading background correction");
    command_serialize(CommandsOpcode::ReadBackgroundCorrection, &[])
}

pub fn read_firmware_version() -> Vec<u8> {
    info!("Reading firmware version");
    command_serialize(CommandsOpcode::FirmwareVersion, &[])
}

pub fn set_baud_rate(baud_rate: u32) -> Vec<u8> {
    info!("Setting baud rate");

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

    command_serialize(CommandsOpcode::BaudRate, &[br as u8, (br >> 8) as u8])
}

pub fn factory_reset() -> Vec<u8> {
    info!("Factory reset");
    command_serialize(CommandsOpcode::FactoryReset, &[])
}

pub fn reboot() -> Vec<u8> {
    info!("Reboot");
    command_serialize(CommandsOpcode::Reboot, &[])
}

pub fn set_bluetooth_on() -> Vec<u8> {
    info!("Bluetooth on");
    command_serialize(CommandsOpcode::Bluetooth, &[0x01, 0x00])
}

pub fn set_bluetooth_off() -> Vec<u8> {
    info!("Bluetooth off");
    command_serialize(CommandsOpcode::Bluetooth, &[0x00, 0x00])
}

pub fn read_mac_address() -> Vec<u8> {
    info!("Get MAC address");
    command_serialize(CommandsOpcode::MacAddress, &[0x01, 0x00])
}

pub fn set_lightsensor_mode(mode: u8, threshold: u8) -> Vec<u8> {
    info!("Set lightsensor mode");
    command_serialize(CommandsOpcode::LightsensorMode, &[mode, threshold])
}

pub fn read_lightsensor_mode() -> Vec<u8> {
    info!("Get lightsensor mode");
    command_serialize(CommandsOpcode::ReadLightsensorMode, &[])
}

// deserialization

#[derive(Debug)]
enum TargetState {
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
pub struct TargetData {
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

fn eat_intraframe(buffer: &[u8]) -> Result<TargetData, ()> {
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

                    TargetData {
                        basic_target_data,
                        engineering_mode_data: Some(eng_data),
                    }
                }
                0x02 => {
                    let basic_target_data = read_basic_target_data(targetdata);

                    TargetData {
                        basic_target_data,
                        engineering_mode_data: None,
                    }
                }
                _ => {
                    error!("Unknown datatype");
                    return Err(());
                }
            };

            let _speed = (*calibration) as i8; //?

            Ok(target_data)
        }
        _ => {
            error!("Intraframe is incorrect");
            Err(())
        }
    }
}

pub fn eat_packet(buffer: &[u8]) -> Result<TargetData, ()> {
    match buffer {
        // received data
        [0xF4, 0xF3, 0xF2, 0xF1, len_l, len_h, intraframe @ .., 0xF8, 0xF7, 0xF6, 0xF5] => {
            let len = u16::from_le_bytes([*len_l, *len_h]);

            assert!(len as usize == intraframe.len());

            eat_intraframe(intraframe)
        }
        _ => {
            error!("Packet is incorrect");
            Err(())
        }
    }
}

#[derive(Debug)]
pub struct Ack {
    command: u16,
    data: Vec<u8>,
}

pub fn eat_ack(buffer: &[u8]) -> Result<Ack, ()> {
    match buffer {
        [0xFD, 0xFC, 0xFB, 0xFA, len_l, len_h, cmd_l, cmd_h, data @ .., 0x04, 0x03, 0x02, 0x01] => {
            let len = u16::from_le_bytes([*len_l, *len_h]);

            assert!(len as usize == data.len() + 2);

            let command_word = u16::from_le_bytes([*cmd_l, *cmd_h]);

            Ok(Ack {
                command: command_word,
                data: data.to_vec(),
            })
        }
        _ => {
            error!("ACK is incorrect");
            Err(())
        }
    }
}
