#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use rp235x_hal::{
    pac,
    clocks::init_clocks_and_plls,
    sio::Sio,
    watchdog::Watchdog,
    cyw43::{Cyw43, PowerMode},
};

#[entry]
fn main() -> ! {
    // -------------------------------
    // Take RP2350 peripherals
    // -------------------------------
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // Default 12 MHz crystal on Pico 2 W
    let _clocks = init_clocks_and_plls(
        12_000_000,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    ).ok().unwrap();

    let sio = Sio::new(pac.SIO);

    // -------------------------------
    // Initialize CYW43 WiFi/LED chip
    // -------------------------------
    let mut cyw43 = Cyw43::new(pac.PIO0, &mut pac.RESETS, sio.gpio_bank0)
        .expect("CYW43 init failed");

    cyw43.set_power_mode(PowerMode::PowerSave).unwrap();

    // -------------------------------
    // Blink onboard LED forever
    // -------------------------------
    loop {
        cyw43.gpio_set(0, true).unwrap();   // LED ON
        delay_ms(500);

        cyw43.gpio_set(0, false).unwrap();  // LED OFF
        delay_ms(500);
    }
}

/// crude busy-loop delay
fn delay_ms(ms: u32) {
    // ~120 MHz / 4 -> approx 30k cycles per iteration
    for _ in 0..(ms * 30_000) {
        cortex_m::asm::nop();
    }
}
