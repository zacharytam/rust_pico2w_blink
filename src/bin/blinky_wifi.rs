#![no_std]
#![no_main]

use cortex_m_rt::entry;
use rp_pico::hal::{
    self,
    pac,
    gpio::Pins,
    pio::{PIOBuilder, SM0, PIO},
    clocks::init_clocks_and_plls,
    sio::Sio,
};
use panic_halt as _;

#[entry]
fn blink() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    // Initialize clocks
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        12_000_000,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    ).ok().unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Use GPIO 25 (onboard LED)
    let led = pins.gpio25.into_push_pull_output();

    // Use PIO0 and SM0
    let (mut pio, sm0, _, _, _) = PIO::new(pac.PIO0, &mut pac.RESETS);

    // Simple PIO program: toggle a pin
    let program = pio_proc::pio_asm!(
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

    // Loop forever (CPU can do other work)
    loop {
        cortex_m::asm::nop();
    }
}
