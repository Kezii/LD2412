use log::error;

use smallvec::SmallVec;

use crate::{RadarDriver, RadarLLFrame};

#[derive(Debug, Clone, Copy)]
pub enum TrackingMode {
    SingleTarget = 0x01,
    MultiTarget = 0x02,
}

#[derive(Debug)]
pub enum Ld2450Command {
    /// Send this command to enable configuration mode, otherwise the radar will ignore all other commands
    EnableConfiguration,
    /// Send this command to end configuration mode, so the radar will start to work
    EndConfiguration,
    /// Set tracking mode to single target
    SingleTargetTracking,
    /// Set tracking mode to multi-target (up to 3 targets)
    MultiTargetTracking,
    /// Query current tracking mode
    QueryTrackingMode,
    /// Read firmware version
    FirmwareVersion,
    /// Set serial port baud rate
    BaudRate(u32),
    /// Restore factory settings
    FactoryReset,
    /// Reboot the module
    Reboot,
    /// Enable Bluetooth
    BluetoothOn,
    /// Disable Bluetooth
    BluetoothOff,
    /// Get MAC address
    MacAddress,
    /// Query the current zone filtering configuration
    QueryZoneFiltering,
    /// Set zone filtering configuration
    /// Type, Region1 (x1,y1,x2,y2), Region2 (x1,y1,x2,y2), Region3 (x1,y1,x2,y2)
    SetZoneFiltering(u16, [(i16, i16, i16, i16); 3]),
}

impl RadarDriver for Ld2450Command {
    fn get_opcode(&self) -> u16 {
        match self {
            Ld2450Command::EnableConfiguration => 0x00FF,
            Ld2450Command::EndConfiguration => 0x00FE,
            Ld2450Command::SingleTargetTracking => 0x0080,
            Ld2450Command::MultiTargetTracking => 0x0090,
            Ld2450Command::QueryTrackingMode => 0x0091,
            Ld2450Command::FirmwareVersion => 0x00A0,
            Ld2450Command::BaudRate(_) => 0x00A1,
            Ld2450Command::FactoryReset => 0x00A2,
            Ld2450Command::Reboot => 0x00A3,
            Ld2450Command::BluetoothOn => 0x00A4,
            Ld2450Command::BluetoothOff => 0x00A4, // Same opcode, different parameter
            Ld2450Command::MacAddress => 0x00A5,
            Ld2450Command::QueryZoneFiltering => 0x00C1,
            Ld2450Command::SetZoneFiltering(_, _) => 0x00C2,
        }
    }

    fn serialize_data(&self, data: &mut SmallVec<[u8; 16]>) {
        match self {
            Ld2450Command::EnableConfiguration => {
                data.extend_from_slice(&[0x01, 0x00]);
            }
            Ld2450Command::EndConfiguration => {}
            Ld2450Command::SingleTargetTracking => {}
            Ld2450Command::MultiTargetTracking => {}
            Ld2450Command::QueryTrackingMode => {}
            Ld2450Command::FirmwareVersion => {}
            Ld2450Command::BaudRate(baud_rate) => {
                let br: u16 = match baud_rate {
                    9600 => 0x0001,
                    19200 => 0x0002,
                    38400 => 0x0003,
                    57600 => 0x0004,
                    115200 => 0x0005,
                    230400 => 0x0006,
                    256000 => 0x0007,
                    460800 => 0x0008,
                    _ => panic!("Unsupported baud rate"),
                };

                data.extend_from_slice(&[br as u8, (br >> 8) as u8]);
            }
            Ld2450Command::FactoryReset => {}
            Ld2450Command::Reboot => {}
            Ld2450Command::BluetoothOn => {
                data.extend_from_slice(&[0x01, 0x00]);
            }
            Ld2450Command::BluetoothOff => {
                data.extend_from_slice(&[0x00, 0x00]);
            }
            Ld2450Command::MacAddress => {
                data.extend_from_slice(&[0x01, 0x00]);
            }
            Ld2450Command::QueryZoneFiltering => {}
            Ld2450Command::SetZoneFiltering(filter_type, regions) => {
                // Add filter type
                data.push(*filter_type as u8);
                data.push((*filter_type >> 8) as u8);

                // Add region data
                for region in regions {
                    // x1 coordinate
                    data.push(region.0 as u8);
                    data.push((region.0 >> 8) as u8);

                    // y1 coordinate
                    data.push(region.1 as u8);
                    data.push((region.1 >> 8) as u8);

                    // x2 coordinate
                    data.push(region.2 as u8);
                    data.push((region.2 >> 8) as u8);

                    // y2 coordinate
                    data.push(region.3 as u8);
                    data.push((region.3 >> 8) as u8);
                }
            }
        }
    }
}

impl Ld2450Command {
    pub fn to_llframe(&self) -> RadarLLFrame {
        let mut data = SmallVec::new();
        self.serialize_data(&mut data);
        RadarLLFrame::CommandAckFrame(self.get_opcode(), data)
    }
}

// Target data structures

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: i16, // mm
    pub y: i16, // mm
}

#[derive(Debug, Clone, Copy)]
pub struct TargetData {
    pub position: Position,
    pub speed: i16,               // cm/s
    pub distance_resolution: u16, // mm
}

#[derive(Debug)]
pub struct Ld2450TargetData {
    pub targets: SmallVec<[TargetData; 3]>,
}

impl Ld2450TargetData {
    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        if buffer.len() < 24 {
            // 3 targets, 8 bytes each
            error!("Buffer too short for LD2450 target data");
            return None;
        }

        let mut targets = SmallVec::new();

        // Process each target (up to 3 targets)
        for i in 0..3 {
            let base_index = i * 8;

            // Check if we're still within bounds
            if base_index + 8 > buffer.len() {
                break;
            }

            // Check if target exists (all zeros means no target)
            let all_zeros = buffer[base_index..base_index + 8].iter().all(|&b| b == 0);
            if all_zeros {
                continue;
            }

            // Extract target data
            // X coordinate
            let mut x = i16::from_le_bytes([buffer[base_index], buffer[base_index + 1]]);
            // Y coordinate
            let mut y = i16::from_le_bytes([buffer[base_index + 2], buffer[base_index + 3]]);
            // Speed
            let mut speed = i16::from_le_bytes([buffer[base_index + 4], buffer[base_index + 5]]);
            // Distance resolution
            let distance = u16::from_le_bytes([buffer[base_index + 6], buffer[base_index + 7]]);

            // Handle sign bit in highest bit for x, y, and speed
            if (buffer[base_index + 1] & 0x80) != 0 {
                x &= 0x7FFF; // Clear sign bit
            } else {
                x = -x; // Negative value
            }

            if (buffer[base_index + 3] & 0x80) != 0 {
                y &= 0x7FFF; // Clear sign bit
            } else {
                y = -y; // Negative value
            }

            if (buffer[base_index + 5] & 0x80) != 0 {
                speed &= 0x7FFF; // Clear sign bit
            } else {
                speed = -speed; // Negative value
            }

            targets.push(TargetData {
                position: Position { x, y },
                speed,
                distance_resolution: distance,
            });
        }

        Some(Ld2450TargetData { targets })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_target_data() {
        // Example data from PDF documentation page 13:
        // AA FF 03 00 0E 03 B1 86 10 00 40 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 55 CC
        // Extracting just the target data (without header/footer):
        let target_data = [
            // Target 1
            0x0E, 0x03, 0xB1, 0x86, 0x10, 0x00, 0x40, 0x01, // Target 2 (not present)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Target 3 (not present)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let result = Ld2450TargetData::deserialize(&target_data);
        assert!(result.is_some());

        let target_data = result.unwrap();
        assert_eq!(
            target_data.targets.len(),
            1,
            "Should parse exactly one target"
        );

        let target = &target_data.targets[0];

        // The PDF example explains:
        // Target 1 X coordinate: 0x0E + 0x03 * 256 = 782, then 0 - 782 = -782 mm (since high bit is 0)
        assert_eq!(target.position.x, -782, "X coordinate should be -782 mm");

        // Target 1 Y coordinate: 0xB1 + 0x86 * 256 = 34481,
        // Since high bit is 1, it's positive: 34481 - 2^15 = 1713 mm
        assert_eq!(target.position.y, 1713, "Y coordinate should be 1713 mm");

        // Target 1 speed: 0x10 + 0x00 * 256 = 16,
        // Since high bit is 0, it's negative: 0 - 16 = -16 cm/s
        assert_eq!(target.speed, -16, "Speed should be -16 cm/s");

        // Target 1 distance resolution: 0x40 + 0x01 * 256 = 320 mm
        assert_eq!(
            target.distance_resolution, 320,
            "Distance resolution should be 320 mm"
        );
    }
}
