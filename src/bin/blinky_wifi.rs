#![no_std]
#![no_main]

use rp235x_hal::{
    pac,
    gpio::{FunctionPio0, Output, PinId, PushPull},
    pio::{PIO, PIOBuilder, SM0},
    clocks::init_clocks_and_plls,
    sio::Sio,
    watchdog::Watchdog,
};
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use cortex_m_rt::entry;
use panic_halt as _;
use embedded_hal::digital::v2::OutputPin;
use cortex_m::asm::nop;

// Include firmware binaries
const FW: &[u8] = include_bytes!("../cyw43-firmware/43439A0.bin");
const CLM: &[u8] = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // Init clocks
    let _clocks = init_clocks_and_plls(
        12_000_000,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);

    // Set up PIO pins
    let pins = rp235x_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let pwr = pins.gpio23.into_push_pull_output();
    let cs = pins.gpio25.into_push_pull_output();
    let miso = pins.gpio24;
    let mosi = pins.gpio29;

    // Set up PIO0, SM0
    let (mut pio, sm0, _, _, _) = PIO::new(pac.PIO0, &mut pac.RESETS);

    // SPI to CYW43 via PIO
    let spi = PioSpi::new(
        &mut pio.common,
        sm0,
        RM2_CLOCK_DIVIDER,
        cs,
        pwr,
        mosi,
        miso,
        pac.DMA_CH0,
    );

    // Initialize CYW43 state
    let mut state = cyw43_pio::State::new();
    let (_net_device, mut control, mut runner) =
        cyw43_pio::new(&mut state, pwr, spi, FW);

    // Initialize CLM
    control.init(CLM);

    // Blink LED forever
    loop {
        control.gpio_set(0, true);
        for _ in 0..10_000 {
            nop();
        }
        control.gpio_set(0, false);
        for _ in 0..10_000 {
            nop();
        }
    }
}
