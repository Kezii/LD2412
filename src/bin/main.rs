//! Duplex example
//!
//! This example tests the ability to clone a serial port. It works by creating
//! a new file descriptor, and therefore a new `SerialPort` object that's safe
//! to send to a new thread.
//!
//! This example selects the first port on the system, clones the port into a child
//! thread that writes data to the port every second. While this is running the parent
//! thread continually reads from the port.
//!
//! To test this, have a physical or virtual loopback device connected as the
//! only port in the system.

use std::io::Write;
use std::time::Duration;
use std::{io, thread};

use log::info;
use plotters::data;
use LD2412::EngineeringModeData;

fn main() {
    env_logger::init();
    // Open the first serialport available.
    let port_name = &serialport::available_ports().expect("No serial port")[0].port_name;
    let mut port = serialport::new(port_name, 115200)
        .open()
        .expect("Failed to open serial port");

    let mut clone = port.try_clone().expect("Failed to clone");

    fn send_command(port: &mut Box<dyn serialport::SerialPort>, command: &[u8]) {
        port.write_all(command)
            .expect("Failed to write to serial port");
        thread::sleep(Duration::from_millis(100));
    }

    thread::spawn(move || {
        return;
        thread::sleep(Duration::from_millis(1000));

        send_command(&mut clone, &LD2412::enable_configuration());

        send_command(&mut clone, &LD2412::read_resolution());

        send_command(
            &mut clone,
            &LD2412::set_resolution(LD2412::RadarResolution::cm25),
        );

        send_command(&mut clone, &LD2412::read_resolution());

        send_command(&mut clone, &LD2412::read_firmware_version());

        send_command(&mut clone, &LD2412::read_mac_address());

        send_command(&mut clone, &LD2412::read_motion_sensitivity());

        send_command(&mut clone, &LD2412::read_static_sensitivity());

        send_command(&mut clone, &LD2412::set_enable_engineering_mode());

        send_command(&mut clone, &LD2412::end_configuration());
    });

    let mut buffer: [u8; 1] = [0; 1];
    let mut pers_buffer = Vec::new();

    port.flush().expect("Failed to flush serial port");

    loop {
        match port.read(&mut buffer) {
            Ok(_) => {
                pers_buffer.extend_from_slice(&buffer);

                //trace!("{:x?}", pers_buffer);

                // data frame format is like
                // F4 F3 F2 F1 len_l len_h data F8 F7 F6 F5
                if pers_buffer.ends_with(&[0xF8, 0xF7, 0xF6, 0xF5]) {
                    let data = LD2412::eat_packet(&pers_buffer);

                    match data {
                        Ok(data) => {
                            println!("{:#?}", data.basic_target_data);
                            if let Some(eng_data) = data.engineering_mode_data {
                                println!("{:#?}", eng_data);
                            }
                        }

                        Err(e) => {
                            eprintln!("{:?}", e);
                        }
                    }

                    pers_buffer.clear();
                }

                // ack frame format is like
                // FD FC FB FA data 04 03 02 01
                if pers_buffer.ends_with(&[0x04, 0x03, 0x02, 0x01]) {
                    let ack = LD2412::eat_ack(&pers_buffer);
                    println!("{:#?}", ack);
                    pers_buffer.clear();
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
