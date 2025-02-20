#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(simple_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use simple_os::println;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    simple_os::test_panic_handler(info);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    simple_os::init();
    x86_64::instructions::interrupts::int3();

    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    };

    #[cfg(test)]
    test_main();

    loop {}
}

#[cfg(test)]
mod test {
    use simple_os::{serial_print, serial_println};

    #[test_case]
    fn trivial_assertion() {
        serial_print!("trivial assertion... ");
        assert_eq!(1, 1);
        serial_println!("[ok]");
    }
}
