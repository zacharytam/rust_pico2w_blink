#![no_std]
#![no_main]

use rp235x_hal::{
    clocks::init_clocks_and_plls,
    pac,
    gpio::{Pins, Level, Output},
    pio::{PIO, PIOBuilder, SM0},
    sio::Sio,
    watchdog::Watchdog,
};
use cortex_m_rt::entry;
use panic_halt as _;
use pio_proc::pio_asm;

#[entry]
fn main() -> ! {
    // Take ownership of the PAC
    let mut pac = pac::Peripherals::take().unwrap();

    // Initialize clocks
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let _clocks = init_clocks_and_plls(
        12_000_000,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    ).ok().unwrap();

    // Initialize SIO and pins
    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

    let led = pins.gpio0.into_push_pull_output(); // Pico 2 W onboard LED (GPIO0)

    // Initialize PIO0 and SM0
    let (mut pio, sm0, _, _, _) = PIO::new(pac.PIO0, &mut pac.RESETS);

    // Simple PIO program: toggle a pin
    let program = pio_asm!(
        ".wrap_target",
        "set pins, 1",
        "nop",
        "set pins, 0",
        "nop",
        ".wrap"
    );
    let installed = pio.install(&program.program).unwrap();

    let mut sm = PIOBuilder::from_program(installed)
        .set_pins(led.id().num, 1)
        .build(sm0);

    sm.start();

    loop {
        cortex_m::asm::nop(); // CPU can do other work
    }
}
