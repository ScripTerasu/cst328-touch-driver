// This example targets the Waveshare ESP32-S3-Touch-LCD-2.8 board (CST328
// touch chip) specifically — see docs/ESP32-S3-Touch-LCD-2.8-schematic.pdf
// at the repository root. The touch controller has its own dedicated I2C bus
// (SDA=GPIO1, SCL=GPIO3) and its own dedicated RST pin (GPIO2), separate from
// both the LCD's SPI/RST pins and the onboard IMU/RTC's I2C bus
// (SCL=GPIO10, SDA=GPIO11) — don't assume any of these are shared on a
// different board; always check your own board's schematic.
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use cst328::CST328;
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use esp_hal::Async;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let _ = spawner;

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let i2c_freq_khz: u32 = 400;

    // TP_SDA/TP_SCL are a dedicated bus for the touch controller on this board — NOT
    // shared with the onboard IMU/RTC bus (SCL=GPIO10, SDA=GPIO11), confirmed against
    // docs/ESP32-S3-Touch-LCD-2.8-schematic.pdf (net labels TP_SDA/TP_SCL on GPIO1/GPIO3)
    // and cross-checked against github.com/zonfacter/ESP32-S3-Touch-LCD-2.8's
    // `PIN_TOUCH_SDA`/`PIN_TOUCH_SCL` constants, which agree exactly.
    let i2c = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(Rate::from_khz(i2c_freq_khz)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO3)
    .into_async();

    // TP_RST is active-low (see the driver README's wiring section), so idle it high —
    // the driver's own reset() pulses it low/high on init(), we just own the pin here.
    // GPIO2 is confirmed against THIS board's schematic (TP_RST net label) and the
    // zonfacter reference repo's `PIN_TOUCH_RST` — it's a dedicated touch RST pin, not
    // shared with the LCD's own RST (GPIO39).
    let rst = Output::new(peripherals.GPIO2, Level::High, OutputConfig::default());

    // TP_INT confirmed as GPIO4 against the board schematic (TP_INT net label) and the
    // zonfacter reference repo's `PIN_TOUCH_INT`/`CST328_INT_PIN`. The CST328 datasheet
    // (section 10.6, "中断方式") says the interrupt edge is configurable (rising or
    // falling) but doesn't say which one this panel's firmware uses, and there's no
    // register in this driver to query or set it — wait for either edge instead of
    // guessing a polarity; worst case is one harmless extra `touches()` read. Pull::Up is
    // a safe default in case the line is open-drain without its own external pull-up.
    let touch_int = Input::new(peripherals.GPIO4, InputConfig::default().with_pull(Pull::Up));

    let driver = CST328::new(i2c, Delay).with_reset(rst);
    spawner.spawn(touch_task(driver, touch_int).unwrap());

    loop {
        info!("Touch controller running");
        Timer::after(Duration::from_secs(60)).await;
    }
}

#[embassy_executor::task]
#[allow(
    clippy::large_stack_frames,
    reason = "clippy sums the whole async state machine (which embassy stores in the static \
    TaskPool, not on the call stack) as if it were the function's stack frame. Verified via \
    objdump on the built xtensa-esp32s3-none-elf binary: the real `poll()` entry frame is 192 \
    bytes, well under the crate's 1024-byte threshold."
)]
async fn touch_task(
    mut touch_driver: CST328<I2c<'static, Async>, Delay, Output<'static>>,
    mut touch_int: Input<'static>,
) {
    // 1. Initialize the driver at task startup (this also pulses the RST pin)
    if let Err(e) = touch_driver.init().await {
        error!("Failed to initialize touch panel: {:?}", e);
        return;
    }

    // 2. Log what init() discovered, so a flashed board tells you what it found instead
    // of just "it works". ChipInfo derives defmt::Format, so this prints every field
    // (chip_id, project_id, resolution, firmware version, firmware CRC/checksum)
    // without hand-picking any of them. model_name() always reads "CST328/CST3530" —
    // this driver can't tell the two chips apart (see the crate README).
    info!(
        "Touch panel ready: {} -> {}",
        touch_driver.model_name(),
        touch_driver.chip_info()
    );

    loop {
        // 3. Sleep until TOUCH_INT actually toggles instead of polling on a fixed
        // interval — the chip only drives it when it has a report ready (see the
        // comment where `touch_int` is created), so this keeps the task idle (and
        // the I2C bus quiet) between touches instead of reading on a fixed interval
        // whether or not anything changed.
        touch_int.wait_for_any_edge().await;

        match touch_driver.touches().await {
            Ok(points) => {
                // `flatten()` filters out `None` and unwraps `Some(Point)` in one pass
                for point in points.iter().flatten() {
                    info!(
                        "Touch detected -> ID: {}, X: {}, Y: {}, pressure: {}",
                        point.track_id, point.x, point.y, point.area
                    );

                    // Send coordinates to your GUI (LVGL, Slint, etc.) or gesture logic
                }
            }
            Err(e) => {
                error!("I2C communication error: {:?}", e);
            }
        }
    }
}
