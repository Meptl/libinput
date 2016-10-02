extern crate libinput;

use libinput::events::Event;

fn main() {
    let mut input = libinput::LibInput::new_from_udev().unwrap();
    for e in input.events() {
        match e {
            Event::DeviceAdd(dev) => println!("Added {} {} {}", dev.name(), dev.physical_seat(), dev.logical_seat()),
            Event::DeviceRemove(dev) => println!("Removed {} {} {}", dev.name(), dev.physical_seat(), dev.logical_seat()),
            Event::KeyboardInput(state, key) => {
                if key == 1 { // Escape key
                    break;
                }
                println!("Keypress {}", key)
            },
            _ => {},
        }
    }
}
