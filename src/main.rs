//! This example toggles the GPIO25 pin, using a PIO program.
//!
//! If a LED is connected to that pin, like on a Pico board, the LED should blink.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use hal::gpio::{FunctionPio0, Pin};
use hal::pac;
use hal::pio::PIOExt;
use hal::Sio;
use panic_halt as _;
use pio::Assembler;
use rp2040_hal as hal;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

enum MorseUnit {
    Short,
    Long,
    NewUnit,
    NewLetter,
    NewWord,
}

fn blip(unit: MorseUnit, a: &mut Assembler<32_usize>) {
    const SHORT: u8 = 4;
    const LONG: u8 = 12;
    const LONG_PAUSE: u8 = 28;
    match unit {
        MorseUnit::Short => a.set_with_delay(pio::SetDestination::PINS, 1, SHORT),
        MorseUnit::Long => a.set_with_delay(pio::SetDestination::PINS, 1, LONG),
        MorseUnit::NewUnit => a.set_with_delay(pio::SetDestination::PINS, 0, SHORT),
        MorseUnit::NewLetter => a.set_with_delay(pio::SetDestination::PINS, 0, LONG),
        MorseUnit::NewWord => a.set_with_delay(pio::SetDestination::PINS, 0, LONG_PAUSE),
    }
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // configure LED pin for Pio0.
    let _led: Pin<_, FunctionPio0> = pins.gpio25.into_mode();
    // PIN id for use inside of PIO
    let led_pin_id = 25;

    // Define some simple PIO program.

    let mut a = pio::Assembler::<32>::new();
    let mut wrap_target = a.label();
    let mut wrap_source = a.label();
    // Set pin as Out
    a.set(pio::SetDestination::PINDIRS, 1);
    // Define begin of program loop
    a.bind(&mut wrap_target);
    blip(MorseUnit::Short, &mut a);
    blip(MorseUnit::NewUnit, &mut a);
    blip(MorseUnit::Short, &mut a);
    blip(MorseUnit::NewUnit, &mut a);
    blip(MorseUnit::Short, &mut a);
    blip(MorseUnit::NewLetter, &mut a);
    blip(MorseUnit::Long, &mut a);
    blip(MorseUnit::NewUnit, &mut a);
    blip(MorseUnit::Long, &mut a);
    blip(MorseUnit::NewUnit, &mut a);
    blip(MorseUnit::Long, &mut a);
    blip(MorseUnit::NewWord, &mut a);
    blip(MorseUnit::Short, &mut a);
    blip(MorseUnit::NewUnit, &mut a);
    blip(MorseUnit::Short, &mut a);
    blip(MorseUnit::NewUnit, &mut a);
    blip(MorseUnit::Short, &mut a);
    blip(MorseUnit::NewLetter, &mut a);
    a.bind(&mut wrap_source);
    // The labels wrap_target and wrap_source, as set above,
    // define a loop which is executed repeatedly by the PIO
    // state machine.
    let program = a.assemble_with_wrap(wrap_source, wrap_target);

    // Initialize and start PIO
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let installed = pio.install(&program).unwrap();
    let div = 0f32; // as slow as possible (0 is interpreted as 65536)
    let (sm, _, _) = rp2040_hal::pio::PIOBuilder::from_program(installed)
        .set_pins(led_pin_id, 1)
        .clock_divisor(div)
        .build(sm0);
    sm.start();

    // PIO runs in background, independently from CPU
    #[allow(clippy::empty_loop)]
    loop {}
}
