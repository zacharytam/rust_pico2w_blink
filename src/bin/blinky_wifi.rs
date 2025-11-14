#![no_std]
#![no_main]

use rp235x_hal::{
    pac,
    gpio::{Pins, Level, Output},
    pio::{PIO, PIOBuilder, SM0},
    sio::Sio,
    clocks::init_clocks_and_plls,
    watchdog::Watchdog,
};
use cortex_m_rt::entry;
use cortex_m;
use pio_proc::pio_asm;
use panic_halt as _;

use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};

// Include CYW43 firmware blobs
const FW: &[u8] = include_bytes!("../../cyw43-firmware/43439A0.bin");
const CLM: &[u8] = include_bytes!("../../cyw43-firmware/43439A0_clm.bin");

#[entry]
fn main() -> ! {
    // Take peripherals
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

    // Initialize SIO and GPIOs
    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

    // Onboard LED
    let mut led = pins.gpio25.into_push_pull_output();

    // Initialize PIO0 and State Machine 0
    let (mut pio, sm0, _, _, _) = PIO::new(pac.PIO0, &mut pac.RESETS);

    // Simple PIO program: toggle LED
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

    // Initialize CYW43 SPI (optional, you can skip if not using Wi-Fi)
    let _pwr = Output::new(pins.gpio23, Level::Low);
    let _cs  = Output::new(pins.gpio25, Level::High);
    let _spi = PioSpi::new(
        &mut pio.common,
        sm0,
        RM2_CLOCK_DIVIDER,
        pio.irq0(),
        _cs,
        pins.gpio24,
        pins.gpio29,
        pac.DMA_CH0,
    );

    // Optional: initialize CYW43 firmware manually (blocking)
    // let mut state = cyw43::State::new();
    // cyw43::new(&mut state, _pwr, _spi, FW);

    // Main loop: toggle LED via CPU (for testing)
    loop {
        led.set_high().unwrap();
        cortex_m::asm::delay(12_000_000); // ~1 second
        led.set_low().unwrap();
        cortex_m::asm::delay(12_000_000);
    }
}
