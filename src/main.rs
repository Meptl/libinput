extern crate libinput_sys;
extern crate libc;

mod ffi;
use ffi::LibInput;

fn main() {
    let mut input = LibInput::new_from_udev().unwrap();
    while input.poll().is_none() {
        let mut event = input.get_event();
        while event.is_some() {
            let e = event.unwrap();
            e.print_header();
            e.print();
            event = input.get_event();
        }
    }
}
