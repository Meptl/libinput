/// User needs to be part of input group to run this program as non-root.
extern crate libinput_sys;
extern crate libc;

pub mod events;
use events::{Device, Event};

use ::libinput_sys::*;
use ::std::os::unix::io::FromRawFd;
use ::std::os::raw::{c_char, c_int, c_void};
use ::std::ffi::CString;
use ::std::fs::File;

unsafe extern "C" fn open_restricted(path: *const c_char, flags: c_int, user_data: *mut c_void) -> c_int {
    // We avoid creating a Rust File because that requires abiding by Rust lifetimes.
    let fd = unsafe { ::libc::open(path, flags) };
    if fd < 0 {
        println!("open_restricted failed.");
    }
    fd
}

unsafe extern "C" fn close_restricted(fd: c_int, user_data: *mut c_void) {
    let f = unsafe { File::from_raw_fd(fd) };
    // File is closed when dropped.
}

fn default_options() -> tools_context {
    let seat_cstr = CString::new("seat0").unwrap();
    let default_options = tools_options {
        backend: tools_backend::BACKEND_UDEV,
        device: std::ptr::null(),
        seat: seat_cstr.as_ptr(),
        grab: 0,

        verbose: 0,
        tapping: -1,
        tap_map: libinput_config_tap_button_map::LIBINPUT_CONFIG_TAP_MAP_LRM,
        drag: -1,
        drag_lock: -1,
        natural_scroll: -1,
        left_handed: -1,
        middlebutton: -1,
        dwt: -1,
        click_method: libinput_config_click_method::LIBINPUT_CONFIG_CLICK_METHOD_NONE,
        scroll_method: libinput_config_scroll_method::LIBINPUT_CONFIG_SCROLL_NO_SCROLL,
        scroll_button: -1,
        speed: 0_f64,
        profile: libinput_config_accel_profile::LIBINPUT_CONFIG_ACCEL_PROFILE_NONE,
    };

    tools_context {
        options: default_options,
        user_data: std::ptr::null_mut(),
    }
}

pub struct LibInput {
    lib_handle: *mut libinput,
    interface: libinput_interface,
}

impl LibInput {
    pub fn new_from_udev() -> Result<LibInput, &'static str> {
        let mut interface = libinput_interface {
            open_restricted: Some(open_restricted),
            close_restricted: Some(close_restricted),
        };

        let udev = unsafe { udev_new() };

        if udev.is_null() {
            return Err("Failed to initialize udev");
        }

        let mut tools_context = default_options();

        let lib_handle = unsafe {
            libinput_udev_create_context(((&mut interface) as *mut _),
            (((&mut tools_context) as *mut tools_context) as *mut _), udev)
        };

        if lib_handle.is_null() {
            return Err("Failed to initialize context with udev");
        }

        let ret = unsafe { libinput_udev_assign_seat(lib_handle, (&tools_context).options.seat) };

        if ret > 0 {
            return Err("Failed to assign seat");
        }

        unsafe { udev_unref(udev) };

        Ok(LibInput {
            lib_handle: lib_handle,
            interface: interface,
        })
    }

    pub fn events(&mut self) -> EventIterator {
        EventIterator::new(self)
    }

    /// Helper functions for creating Events
    fn create_keyboard_event(event_handle: *mut libinput_event) -> Event {
        // This function is simply a type  cast in C
        let key_event = unsafe { libinput_event_get_keyboard_event(event_handle) };
	    let key_state = unsafe { libinput_event_keyboard_get_key_state(key_event) };
	    let key = unsafe { libinput_event_keyboard_get_key(key_event) };

        let state = match key_state {
            libinput_key_state::LIBINPUT_KEY_STATE_PRESSED => events::State::Pressed,
            libinput_key_state::LIBINPUT_KEY_STATE_RELEASED => events::State::Released,
        };

        Event::KeyboardInput(state, key)
    }
}

impl Drop for LibInput {
    fn drop(&mut self) {
        // Return value ignored here.
        // This segfaults after polling for events.
        unsafe { libinput_unref(self.lib_handle); }
    }
}

/// An iterator over libinput events.
/// next() blocks until next input.
pub struct EventIterator<'a> {
    handle: &'a mut LibInput,
    pollfd: ::libc::pollfd,
}

impl<'a> EventIterator<'a> {
    pub fn new(input: &mut LibInput) -> EventIterator {
        unsafe { libinput_get_fd(input.lib_handle) };
        let pollfd = ::libc::pollfd {
            fd: 3,
            events: ::libc::POLLIN,
            revents: 0,
        };

        EventIterator {
            handle: input,
            pollfd: pollfd,
        }
    }

    fn event_from(event_handle: *mut libinput_event) -> Event {
        let event_type = unsafe { libinput_event_get_type(event_handle) };
        let event = match event_type{
            libinput_event_type::LIBINPUT_EVENT_NONE => {
                unsafe { abort() };
                Event::None
            },
            libinput_event_type::LIBINPUT_EVENT_DEVICE_ADDED => {
                Event::DeviceAdd(Device::from(event_handle))
            },
            libinput_event_type::LIBINPUT_EVENT_DEVICE_REMOVED => {
                Event::DeviceRemove(Device::from(event_handle))
            },
            libinput_event_type::LIBINPUT_EVENT_KEYBOARD_KEY => {
                LibInput::create_keyboard_event(event_handle)
            },
            libinput_event_type::LIBINPUT_EVENT_POINTER_MOTION => { Event::None },
            libinput_event_type::LIBINPUT_EVENT_POINTER_MOTION_ABSOLUTE => { Event::None },
            libinput_event_type::LIBINPUT_EVENT_POINTER_BUTTON => { Event::None },
            libinput_event_type::LIBINPUT_EVENT_POINTER_AXIS => { Event::None },
            _ => {
                println!("Event type implemented.");
                Event::None
            },
        };

        unsafe { libinput_event_destroy(event_handle) };

        event
    }
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        unsafe { libinput_dispatch((*self).handle.lib_handle) };
        let event = unsafe { libinput_get_event((*self).handle.lib_handle) };

        // No events left, poll file descriptor for more events.
        if event.is_null() {
            let ret = unsafe { ::libc::poll((&mut ((*self).pollfd)) as *mut _, 1, -1) };
            if ret <= -1 {
                return None;
            }
            return self.next();
        }

        // event_from free's the handle.
        let result = EventIterator::event_from(event);

        Some(result)
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
#[derive(Debug)]
pub enum tools_backend { BACKEND_DEVICE = 0, BACKEND_UDEV = 1, }

#[repr(C)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct tools_options {
    pub backend: tools_backend,
    pub device: *const c_char, // if BACKEND_DEVICE
    pub seat: *const c_char,   // if BACKEND_UDEV
    pub grab: c_int,

    pub verbose: c_int,
    pub tapping: c_int,
    pub drag: c_int,
    pub drag_lock: c_int,
    pub natural_scroll: c_int,
    pub left_handed: c_int,
    pub middlebutton: c_int,
    pub click_method: libinput_config_click_method,
    pub scroll_method: libinput_config_scroll_method,
    pub tap_map: libinput_config_tap_button_map,
    pub scroll_button: c_int,
    pub speed: f64,
    pub dwt: c_int,
    pub profile: libinput_config_accel_profile,
}

#[repr(C)]
pub struct tools_context {
    pub options: tools_options,
    pub user_data: *mut c_void,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
