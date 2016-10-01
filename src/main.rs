extern crate libinput;

fn main() {
    println!("Starting.");
    let mut input = libinput::LibInput::new_from_udev().unwrap();
    while input.poll().is_none() {
        let mut event = input.get_event();
        println!("Got event.");
        if event.is_none() {
            println!("None.");
        }
        while event.is_some() {
            let e = event.unwrap();
            e.print_header();
            e.print();
            event = input.get_event();
        }
    }
    println!("Done.");
}
