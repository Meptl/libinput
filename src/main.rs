extern crate libinput;

use libinput::events::{Event, EventType};

fn main() {
    let mut input = libinput::LibInput::new_from_udev().unwrap();
    for e in input.events() {
        let dev = e.device();
        print!("{} {} {} ", dev.name(), dev.physical_seat(), dev.logical_seat());
        match e.event_type() {
            EventType::KeyboardInput(state, key) => {
                if key == 1 { // Escape key
                    break;
                }
                print!("Keypress {} ", key)
            },
            _ => {},
        };
        println!("{}", e.time());
    }
}
