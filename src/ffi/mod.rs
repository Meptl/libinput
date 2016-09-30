use ::libinput_sys::*;
use ::std;
use ::std::os::unix::io::FromRawFd;
use ::std::os::raw::{c_char, c_int, c_void};
use ::std::ffi::CString;
use ::std::ffi::CStr;
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

// Returns an allocated String. Note that the underlying C buffer may be
// deallocated before String
unsafe fn cbuf_to_str(buf: *const c_char) -> String {
    let u8_buf = CStr::from_ptr(buf).to_bytes();
    // to_owned() allocates the buffer.
    std::str::from_utf8(u8_buf).unwrap().to_owned()
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
    pollfd: ::libc::pollfd
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

        let pollfds = ::libc::pollfd {
            fd: unsafe { libinput_get_fd(lib_handle) },
            events: ::libc::POLLIN,
            revents: 0,
        };

        Ok(LibInput {
            lib_handle: lib_handle,
            interface: interface,
            pollfd: pollfds
        })
    }

    pub fn get_fd(&self) -> File {
        let file;
        unsafe {
            let ret = libinput_get_fd(self.lib_handle);
            file = File::from_raw_fd(ret);
        }
        file
    }

    pub fn poll(&mut self) -> Option<&'static str>{
        let ret = unsafe { ::libc::poll((&mut ((*self).pollfd)) as *mut _, 1, -1) };
        if ret <= -1 {
            return Some("Failed to poll.");
        }

        None
    }

    pub fn get_event(&self) -> Option<LibInputEvent> {
        unsafe { libinput_dispatch((*self).lib_handle as *mut _) };
        let event = unsafe { libinput_get_event((*self).lib_handle as *mut _) };
        if event.is_null() {
            None
        } else {
            Some(LibInputEvent::new_from(event))
        }
    }

    /*
    pub fn into_iter(self) -> EventIterator {
        EventIterator::new(self)
    }
    */
}

impl Drop for LibInput {
    fn drop(&mut self) {
        // Return value ignored here.
        unsafe { libinput_unref(self.lib_handle); }
    }
}

pub struct LibInputEvent {
    handle: *mut libinput_event,
}

impl LibInputEvent {
    pub fn new_from(event: *mut libinput_event) -> LibInputEvent {
        LibInputEvent {
            handle: event,
        }
    }

    fn get_dev(&self) -> *mut libinput_device {
        unsafe { libinput_event_get_device((*self).handle) }
    }

    fn get_name(&self) -> String {
        unsafe { cbuf_to_str(libinput_device_get_sysname(self.get_dev())) }
    }

    fn get_type(&self) -> libinput_event_type {
        unsafe { libinput_event_get_type((*self).handle) }
    }

    pub fn print_header(&self) {
        println!("{}", self.get_name());
    }

    pub fn print(&self) {
        match self.get_type() {
            libinput_event_type::LIBINPUT_EVENT_NONE => unsafe { abort() },
            libinput_event_type::LIBINPUT_EVENT_DEVICE_ADDED => println!("Add device"),
            libinput_event_type::LIBINPUT_EVENT_DEVICE_REMOVED => println!("Remove device"),
            libinput_event_type::LIBINPUT_EVENT_KEYBOARD_KEY => println!("Keyboard key"),
            libinput_event_type::LIBINPUT_EVENT_POINTER_MOTION => println!("Pointer motion"),
            libinput_event_type::LIBINPUT_EVENT_POINTER_MOTION_ABSOLUTE => println!("Pointer motion absolute"),
            libinput_event_type::LIBINPUT_EVENT_POINTER_BUTTON => println!("Pointer button"),
            libinput_event_type::LIBINPUT_EVENT_POINTER_AXIS => println!("Pointer axis"),
            _ => println!("Not implemented."),
        };
    }
}

impl Drop for LibInputEvent {
    fn drop(&mut self) {
        unsafe { libinput_event_destroy((*self).handle) };
    }
}

// Iterator blocks until next event.
/*
pub struct EventIterator {
    handle: LibInput,
    curr: *mut libinput_event,
    pollfd: ::libc::pollfd,
}

impl EventIterator {
    pub fn new(input: LibInput) -> EventIterator {
        let pollfd = ::libc::pollfd {
            fd: unsafe { libinput_get_fd(input.lib_handle) },
            events: ::libc::POLLIN,
            revents: 0,
        };

        let ret = unsafe { ::libc::poll((&mut ((*self).pollfd)) as *mut _, 1, -1) };
        if ret <= -1 {
            return None;
        }

        unsafe { libinput_dispatch((*self).handle.lib_handle as *mut _) };
        EventIterator {
            handle: input,
            curr: unsafe { libinput_get_event((*self).handle.lib_handle as *mut _) },
            pollfd: pollfd
        }
    }
}

impl Iterator for EventIterator {
    type Item = LibInputEvent;
    fn next(&mut self) -> Option<LibInputEvent> {
        if self.curr.is_null() {
            let ret = unsafe { ::libc::poll((&mut ((*self).pollfd)) as *mut _, 1, -1) };
            if ret <= -1 {
                return None;
            }
            unsafe { libinput_dispatch((*self).handle.lib_handle as *mut _) };
            let event = unsafe { libinput_get_event((*self).handle.lib_handle as *mut _) };
            self.curr = event;
            return Some(LibInputEvent::new_from(event));
        }

        unsafe { libinput_dispatch((*self).handle.lib_handle as *mut _) };
        let next = unsafe { libinput_get_event((*self).handle.lib_handle as *mut _) };
    }
}
*/

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
