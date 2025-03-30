#![no_std]

use log::warn;
use smallvec::SmallVec;

pub mod ld2412;
pub mod ld2450;

pub trait RadarDriver {
    fn get_opcode(&self) -> u16;
    fn serialize_data(&self, data: &mut SmallVec<[u8; 16]>);
}

#[derive(Debug)]
pub enum RadarLLFrame {
    /// Command and Acknowledgement frame, LD2412 and LD2450
    CommandAckFrame(u16, SmallVec<[u8; 16]>),
    /// LD2412 1D target data
    TargetFrame(SmallVec<[u8; 32]>),
    /// LD2450 2D target data
    TargetFrame2D(SmallVec<[u8; 32]>),
}

impl RadarLLFrame {
    pub fn serialize(&self) -> SmallVec<[u8; 32]> {
        match self {
            RadarLLFrame::CommandAckFrame(opcode, data) => {
                let mut buffer = SmallVec::new();

                buffer.extend_from_slice(&[0xFD, 0xFC, 0xFB, 0xFA]);
                buffer.extend_from_slice(&(data.len() as u16 + 2).to_le_bytes());
                buffer.extend_from_slice(&opcode.to_le_bytes());
                buffer.extend_from_slice(data);
                buffer.extend_from_slice(&[0x04, 0x03, 0x02, 0x01]);

                buffer
            }
            _ => {
                panic!(
                    "you are not supposed to serialize target data, it is only for deserialization"
                )
            }
        }
    }

    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        match buffer {
            [0xFD, 0xFC, 0xFB, 0xFA, len_l, len_h, opcode_l, opcode_h, data @ .., 0x04, 0x03, 0x02, 0x01] =>
            {
                let len = u16::from_le_bytes([*len_l, *len_h]);

                assert!(len as usize == data.len() + 2);

                let opcode = u16::from_le_bytes([*opcode_l, *opcode_h]);

                Some(RadarLLFrame::CommandAckFrame(
                    opcode,
                    SmallVec::from_slice(data),
                ))
            }

            [0xF4, 0xF3, 0xF2, 0xF1, len_l, len_h, intraframe @ .., 0xF8, 0xF7, 0xF6, 0xF5] => {
                let len = u16::from_le_bytes([*len_l, *len_h]);

                if len as usize != intraframe.len() {
                    warn!("Intraframe length is incorrect");

                    return None;
                }

                Some(RadarLLFrame::TargetFrame(SmallVec::from_slice(intraframe)))
            }

            [0xAA, 0xFF, 0x03, 0x00, intraframe @ .., 0x55, 0xCC] => Some(
                RadarLLFrame::TargetFrame2D(SmallVec::from_slice(intraframe)),
            ),

            _ => None,
        }
    }
}
