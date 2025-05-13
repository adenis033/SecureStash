# SecureStash

Creating a secure safe box with Rust on Raspberry Pi Pico.

## Description

With this project, my goal is to craft a secure safebox using the Raspberry Pi Pico. Access to the safebox is made easy with a keypad, where users put in their unique PIN code for authentication. When the correct code is entered, the safebox unlocks, greeted by a friendly green LED and a message on the display. However, if the PIN code is off, a red LED signals a denial of access, accompanied by a buzzer alert. This setup ensures both security and user-friendly feedback, making it a reliable option for protecting valuables.

## Hardware

In my project, the Raspberry Pi Pico microcontroller serves as the central processing unit, mandated for the task. Its low power consumption ensures efficiency throughout. I've also integrated a 4x4 Matrix Keypad for user input, a servo motor for secure locking, and a buzzer for audible alarms. Each component is chosen with care to ensure a dependable system for safeguarding valuables.

## How to run

  Firstly open terminal or cmd and move to the project directory with the command `cd project` and the in terminal/cmd type `cargo build`

## How it works

  Using Rust Programming language, my Rasberry Pi Pico-based system uses a 4x4 matrix to introduce a code to unlock a door with the help of a servomotor.

  We have 2 codes "123A" for opening the door "456B" for closing it and any other code is for the alarm based on the else statement.  

**Declaring the pins for the matrix,LEDs and buzzer**
```rust
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
```
**The loop in which i read from the matrix**

```rust
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
```
**If/Else statments for the code so that we can use the motoservo based on codes that I introduce**

```rust

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
```

**Photo of the final product**

![](poza.jpg)

## Links


1. [Project of a student from past years](https://ocw.cs.pub.ro/courses/pm/prj2022/arosca/rfid-lock)
2. [Door Lock](https://www.youtube.com/watch?v=kGyQS3B1IwU&t=19s&ab_channel=SriTuHobby)
3. [Anti-theft lock](https://www.youtube.com/watch?v=Jg0W165iHYk&t=32s&ab_channel=svsembedded)
