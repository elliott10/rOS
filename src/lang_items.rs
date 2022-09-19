use core::panic::PanicInfo;
use super::backtrace::print_backtrace;

#[panic_handler]
fn panic(info: &PanicInfo) -> !{
	println!("{}", info);
    //print_backtrace();
	loop{}
}

#[no_mangle]
pub extern "C" fn abort() -> !{
	panic!("abort!");
}

