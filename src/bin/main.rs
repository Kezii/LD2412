use std::io::Write;
use std::time::Duration;
use std::{io, thread};

use log::{debug, info, warn};
use LD24xx::ld2412::{Ld2412Command, Ld2412TargetData, RadarResolution};
use LD24xx::ld2450::Ld2450TargetData;
use LD24xx::RadarLLFrame;

fn main() {
    env_logger::init();

    //let baud_rate = 115200; // LD2412 baud rate
    let baud_rate = 256000; // LD2450 baud rate

    // Open the first serialport available.
    let port_name = &serialport::available_ports().expect("No serial port")[0].port_name;
    let mut port = serialport::new(port_name, baud_rate)
        .open()
        .expect("Failed to open serial port");

    let mut clone = port.try_clone().expect("Failed to clone");

    fn send_command(port: &mut Box<dyn serialport::SerialPort>, command: &[u8]) {
        debug!("Sending command: {:x?}", command);
        port.write_all(command)
            .expect("Failed to write to serial port");

        port.flush().expect("Failed to flush serial port");
        thread::sleep(Duration::from_millis(100));
    }

    // thread::spawn(move || {
    //     thread::sleep(Duration::from_millis(1000));

    //     send_command(
    //         &mut clone,
    //         &Ld2412Command::EnableConfiguration.to_llframe().serialize(),
    //     );
    //     thread::sleep(Duration::from_millis(100));

    //     // send_command(
    //     //     &mut clone,
    //     //     &Ld2412Command::Resolution(RadarResolution::Cm25)
    //     //         .to_llframe()
    //     //         .serialize(),
    //     // );

    //     // send_command(
    //     //     &mut clone,
    //     //     &Ld2412Command::FirmwareVersion.to_llframe().serialize(),
    //     // );

    //     send_command(
    //         &mut clone,
    //         &Ld2412Command::EngineeringModeOn.to_llframe().serialize(),
    //     );
    //     thread::sleep(Duration::from_millis(100));

    //     send_command(
    //         &mut clone,
    //         &Ld2412Command::EndConfiguration.to_llframe().serialize(),
    //     );
    // });

    let mut buffer: [u8; 1] = [0; 1];
    let mut pers_buffer = Vec::new();

    port.flush().expect("Failed to flush serial port");

    loop {
        match port.read(&mut buffer) {
            Ok(_) => {
                //debug!("{:x?}", buffer);
                pers_buffer.extend_from_slice(&buffer);
                if pers_buffer.len() > 100 {
                    warn!("overrun, clearing");
                    port.read_to_end(&mut pers_buffer).ok();
                    pers_buffer.clear();
                }

                let frame = RadarLLFrame::deserialize(&pers_buffer);

                if let Some(frame) = frame {
                    info!("{:x?}", frame);

                    match frame {
                        RadarLLFrame::CommandAckFrame(opcode, data) => {
                            info!("{:x?} {:?}", opcode, data);
                        }
                        RadarLLFrame::TargetFrame(data) => {
                            let data = Ld2412TargetData::deserialize(&data);
                            if let Some(data) = data {
                                info!("{:#?}", data.basic_target_data);
                                if let Some(eng_data) = data.engineering_mode_data {
                                    info!("{:#?}", eng_data);
                                }
                            }
                        }
                        RadarLLFrame::TargetFrame2D(data) => {
                            let data = Ld2450TargetData::deserialize(&data);
                            if let Some(data) = data {
                                info!("{:#?}", data);
                            }
                        }
                    }

                    pers_buffer.clear();
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
