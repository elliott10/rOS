use crate::sbi;
use core::fmt::{ self, Write };

pub fn getchar() -> Option<u8> {
    let c = sbi::console_getchar();
    if c < 0 {
        None
    }else{
        Some(c as u8)
    }
}

/////////
pub fn putchar(ch: char){
	sbi::console_putchar(ch as u8 as usize);
}

pub fn puts(s: &str){
	for ch in s.chars(){
		putchar(ch);
	}
}

struct Stdout;

impl fmt::Write for Stdout {
	fn write_str(&mut self, s: &str) -> fmt::Result {
        /*
        use core::fmt::Write;
        write!(crate::uart::Uart::new(0x02500000), "{}", s);
        */
		puts(s);
		Ok(())
	}
}

pub fn _print(args: fmt::Arguments) {
	Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
	($($arg:tt)*) => ({
		$crate::io::_print(format_args!($($arg)*));
	});
}

#[macro_export]
macro_rules! println {
	() => ($crate::print!("\n"));
	($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
