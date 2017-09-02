#![feature(core_panic, intrinsics, lang_items)]

#![no_main]
#![no_std]

#![feature(compiler_builtins_lib)]
extern crate compiler_builtins;

use core::prelude::*;
//use core::panicking;

// Declare some intrinsic functions that are provided to us by the compiler.
//extern "rust-intrinsic" {
//	fn overflowing_add<T>(a: T, b: T) -> T;
//	fn u32_sub_with_overflow(x: u32, y: u32) -> (u32, bool);
//}

#[lang="panic_fmt"]
extern fn panic_fmt(_: ::core::fmt::Arguments, _: &'static str, _: u32) -> ! {
    loop {}
}

//#[lang = "panic_fmt"]
//pub extern fn panic_fmt() { loop {} }

//#[lang = "stack_exhausted"]
//pub extern fn stack_exhausted() { loop {} }

#[lang = "eh_personality"]
pub extern fn eh_personality() { loop {} }


// The following are some very basic marker traits that all Rust programs need
// to function. Normally basic stuff like this is taken care for us in Rust's
// core library, but since we're not using that yet, we need to define it
// ourselves.
//#[lang = "copy"]
//trait Copy {}
//#[lang = "sized"]
//trait Sized {}
//#[lang = "sync"]
//trait Sync {}

// I'm not 100% sure what this function does, but references to it are compiled
// into the program by the Rust compiler. I think it would be called in the case
// of a program panic.
#[no_mangle]
pub fn __aeabi_unwind_cpp_pr0() {
	loop {}
}

#[no_mangle]
pub fn __aeabi_unwind_cpp_pr1() {
    loop {}
}

// This is the top of the stack, as provided to us by the linker.
#[allow(safe_extern_statics)]
extern {
	static _estack: u32;
}


// This is a partial definition of the vector table. It only defines the first
// two entries which, as far as I can tell, are the minimum needed for a program
// to work at all.
// Space for the other interrupt handlers is reserved. I'm not sure if this is
// necessary, but I can imagine that the vector table not having the right
// length could cause all kinds of problems (imagine if it was too short, and
// the linker would place something else directly after it).
pub struct VectorTable {
	pub initial_stack_pointer_value: &'static u32,
	pub reset_handler              : fn(),

	pub other_interrupt_vectors: [u32; 44],
}

unsafe impl Sync for VectorTable {}


// The vector table. We're using GCC-specific functionality to place this into
// the .vectors section, not where it would normally go (I suppose .rodata).
// The linker script makes sure that the .vectors section is at the right place.
#[link_section=".vectors"]
#[allow(safe_extern_statics)]
pub static VECTOR_TABLE: VectorTable = VectorTable {
	initial_stack_pointer_value: &_estack,
	reset_handler              : start,
	other_interrupt_vectors    : [0; 44],
};


// Addresses of several registers used to control parallel I/O.
const PORTA_BASE : *mut u32 = 0x41004400 as *mut u32;
const PORTA_DIRSET : *mut u32 = 0x41004408 as *mut u32;
const PORTA_PINCFG17 : *mut u8 = 0x41004451 as *mut u8;
const PORTA_OUTCLR : *mut u32 = 0x41004414 as *mut u32;
const PORTA_OUTSET : *mut u32 = 0x41004418 as *mut u32;

const PIN17_MASK: u32 = 0x00020000;
const PINCFG_INEN: u8 = 2;


// const PB_PIO_ENABLE       : *mut u32 = 0x400E1000 as *mut u32;
//const PB_OUTPUT_ENABLE    : *mut u32 = 0x400E1010 as *mut u32;
//const PB_SET_OUTPUT_DATA  : *mut u32 = 0x400E1030 as *mut u32;
//const PB_CLEAR_OUTPUT_DATA: *mut u32 = 0x400E1034 as *mut u32;
//
//// Bit mask for PB27. This is pin 13 (the built-in LED) on the Arduino Due.
//const PB27_MASK: u32 = 0x08000000;
//



// This function is the entry point for our application and the handler function
// for the reset interrupt.
fn start() {
	// TODO: This function doesn't copy the .relocate segment into RAM, as init
	//       code would normally do. We're getting away with this, because this
	//       program doesn't use any global variables (or more generally,
	//       doesn't have anything that would go into the .data section). Please
	//       be aware that what might be mistaken for global variables in this
	//       file are actually global constants, which go into the .rodata
	//       section. The problem is that if there were global variables, their
	//       initial value would not be set and the program would just fail
	//       silently. I see two solutions for this:
	//       1. Decide that we're never going to use global variables and remove
	//          support for the .relocate section from the linker script. I
	//          think if that were done, an attempted write to a global variable
	//          might fail the program outright, because the global variable
	//          would reside in ROM. This is just speculation, however, so more
	//          research is required before implementing this solution.
	//       2. Just copy the .relocate segment like any sane microcontroller
	//          program would do. This would definitely be a safe solution, and
	//          the only reason I'm not doing it right now is that it reeks of
	//          cargo cult. I'd rather be bitten from not doing it and then have
	//          a good understanding of why I'm doing it afterwards, than just
	//          do it from the start without really understanding the reason.
	// TODO: This function doesn't initialize the .bss segment to zero, as init
	//       code would normally do. This doesn't make any difference right now,
	//       because there are no uninitialized global variables in this
	//       program. I'm wary of just doing it, for two reasons:
	//       1. I'm not sure why it needs to be done at all. According to my
	//          understanding, C doesn't guarantee that variables are
	//          initialized with any given value, so why should global variables
	//          be different?
	//       2. Even if there is a good reasons (as there probably is), I don't
	//          think global variables are such a hot idea, so I don't want to
	//          do anything that supports them, out of pure stubbornness.

	unsafe {
		// Enable PA17 (pin 13) and configure it for output.
		*PORTA_PINCFG17 = PINCFG_INEN;
		*PORTA_DIRSET = PIN17_MASK;

		// Continuously set and clear output on PB27 (pin 13). This blinks the
		// Due's built-in LED, which is the single purpose of this program.
		loop {
			*PORTA_OUTSET = PIN17_MASK;
            for x in 0..200 {}
			*PORTA_OUTCLR = PIN17_MASK;
            for x in 0..400 {}
		}
	}
}
