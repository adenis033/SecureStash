#![no_std]
#![no_main]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)]

use core::cell::RefCell;
use core::convert::TryFrom;
use core::panic::PanicInfo;
use embassy_executor::Spawner;
use embassy_futures::select::Either4::{First as First4, Second as Second4, Third as Third4, Fourth as Fourth4};
use embassy_futures::select::select4;
use heapless::String;
use core::fmt::Write;

// GPIO
use embassy_rp::gpio::{
    AnyPin, self, Input, Pull, Pin, Level, Output,
};

// Peripherals
use embassy_rp::peripherals::{
    PIN_0, PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12,
    PIN_13, PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25, PIN_26, PIN_27, PIN_28, ADC, SPI0, SPI1, DMA_CH0, USB
};

// PWM
use embassy_rp::pwm::{Config as PwmConfig, Pwm};

// USB
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_rp::bind_interrupts;
use log::info;

// Channel
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};

use embassy_rp::adc::{Adc, Async, Channel as ChannelADC, Config as ConfigADC, InterruptHandler as InterruptHandlerADC};

// Timer
use embassy_time::{Delay, Timer, Duration};

// LCD
use embassy_rp::i2c::{I2c, Config as I2cConfig};
use ag_lcd::{Cursor, LcdDisplay};
use port_expander::dev::pcf8574::Pcf8574;

use core::future::Future;

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    ADC_IRQ_FIFO => InterruptHandlerADC;
});

const DISPLAY_FREQ: u32 = 200_000;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    // Start the serial port over USB driver
    let driver = Driver::new(peripherals.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // Initiate SDA and SCL pins
    let sda = peripherals.PIN_14;
    let scl = peripherals.PIN_15;

    // Initiate Delay
    let delay = Delay;

    // I2C Config
    let mut config = I2cConfig::default();
    config.frequency = DISPLAY_FREQ;

    // Initiate I2C
    let i2c = I2c::new_blocking(peripherals.I2C1, scl, sda, config.clone());
    let mut i2c_expander = Pcf8574::new(i2c, true, true, true);

    // Initiate LCD
    let mut lcd: LcdDisplay<_, _> = LcdDisplay::new_pcf8574(&mut i2c_expander, delay)
        .with_cursor(Cursor::Off)
        .with_reliable_init(10000)
        .build();

    let mut config_pwm: PwmConfig = Default::default();
    let mut pwm_output = Output::new(peripherals.PIN_2, Level::Low);

    let mut column_1 = Output::new(peripherals.PIN_19, Level::Low);
    let mut column_2 = Output::new(peripherals.PIN_18, Level::Low);
    let mut column_3 = Output::new(peripherals.PIN_17, Level::Low);
    let mut column_4 = Output::new(peripherals.PIN_16, Level::Low);
    let mut row_1 = Input::new(peripherals.PIN_26, Pull::Up);
    let mut row_2 = Input::new(peripherals.PIN_22, Pull::Up);
    let mut row_3 = Input::new(peripherals.PIN_21, Pull::Up);
    let mut row_4 = Input::new(peripherals.PIN_20, Pull::Up);

    let mut buzzer = Output::new(peripherals.PIN_1, Level::Low);
    let mut green_led = Output::new(peripherals.PIN_0, Level::Low);
    let mut red_led = Output::new(peripherals.PIN_3, Level::Low);

    let mut text = String::<64>::new();
    write!(text, "Door Unlocked").unwrap();
    let mut text2 = String::<64>::new();
    write!(text2, "Wrong Key").unwrap();
    let mut text3 = String::<64>::new();
    write!(text3, "Door Locked").unwrap();
    let mut textt = String::<64>::new();    write!(textt, "Insert Key:").unwrap();

    async fn move_servo_180_degrees(pwm_output: &mut Output<'static, PIN_2>) {
        pwm_output.set_high();
        Timer::after(Duration::from_millis(2)).await;
        Timer::after(Duration::from_micros(300)).await; // Adding 0.2 ms to achieve closer to 180 degrees
        pwm_output.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
    
    async fn move_servo_back_180_degrees(pwm_output: &mut Output<'static, PIN_2>) {
        pwm_output.set_high();
        Timer::after(Duration::from_millis(1)).await; // 1 ms pulse for 0 degrees
        pwm_output.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
    
    

    fn map_button(column_index: usize, row_index: usize) -> &'static str {
        let map: [[&str; 4]; 4] = [
            ["1", "2", "3", "A"],
            ["4", "5", "6", "B"],
            ["7", "8", "9", "C"],
            ["*", "0", "#", "D"],
        ];
        return map[row_index][column_index];
    }

    loop {
        let mut keypad_code: String<8> = String::try_from("").unwrap();

        while keypad_code.len() < 4 {

            // Set all columns high initially
            column_1.set_high();
            column_2.set_high();
            column_3.set_high();
            column_4.set_high();

            // Scan each column
            for column_index in 0..4 {
                // Set the current column low
                match column_index {
                    0 => column_1.set_low(),
                    1 => column_2.set_low(),
                    2 => column_3.set_low(),
                    3 => column_4.set_low(),
                    _ => unreachable!(),
                }

                // Introduce a small delay to allow the column to settle
                Timer::after(Duration::from_millis(10)).await;

                // Check each row for a button press
                if row_1.is_low() {
                    let button = map_button(column_index, 0);
                    keypad_code.push_str(button).unwrap();
                    info!("Column {}, Row 0 pressed: {}", column_index, button);
                } else if row_2.is_low() {
                    let button = map_button(column_index, 1);
                    keypad_code.push_str(button).unwrap();
                    info!("Column {}, Row 1 pressed: {}", column_index, button);
                } else if row_3.is_low() {
                    let button = map_button(column_index, 2);
                    keypad_code.push_str(button).unwrap();
                    info!("Column {}, Row 2 pressed: {}", column_index, button);
                } else if row_4.is_low() {
                    let button = map_button(column_index, 3);
                    keypad_code.push_str(button).unwrap();
                    info!("Column {}, Row 3 pressed: {}", column_index, button);
                }

                let mut combined_text = String::<128>::new();
                write!(combined_text, "{} {}", textt, keypad_code).unwrap();
                lcd.clear();
                lcd.print(&combined_text);  

                // Reset the column to high
                match column_index {
                    0 => column_1.set_high(),
                    1 => column_2.set_high(),
                    2 => column_3.set_high(),
                    3 => column_4.set_high(),
                    _ => unreachable!(),
                }

                // Debounce delay
                Timer::after(Duration::from_millis(50)).await;

                // Check if the code has reached the desired length
                if keypad_code.len() >= 4 {
                    break;
                }
            }
        }

        info!("Code entered: {}", keypad_code);

        if keypad_code == "123A" {
            info!("Correct code entered, moving servo to 120 degrees.");
            move_servo_180_degrees(&mut pwm_output).await;
            green_led.set_high();
            lcd.clear();
            lcd.print(&text);
            Timer::after(Duration::from_secs(5)).await;
            green_led.set_low();
        } else if keypad_code == "456B" {
            info!("Correct code entered, moving servo back 120 degrees.");
            move_servo_back_180_degrees(&mut pwm_output).await;
            green_led.set_high();
            lcd.clear();
            lcd.print(&text3);
            Timer::after(Duration::from_secs(2)).await;
            green_led.set_low();
        } else {
            move_servo_back_180_degrees(&mut pwm_output).await;
            buzzer.set_high();
            red_led.set_high();
            lcd.clear();
            lcd.print(&text2);
            Timer::after(Duration::from_secs(2)).await;
            red_led.set_low();
            buzzer.set_low();
        }

    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
