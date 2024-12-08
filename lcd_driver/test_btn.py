import RPi.GPIO as GPIO
import time

# GPIO pins for the buttons
BUTTON_PINS = {
    22: "Button 22",
    23: "Button 23",
    24: "Button 24",
    25: "Button 25",
}

# GPIO setup
GPIO.setmode(GPIO.BCM)
for pin in BUTTON_PINS:
    GPIO.setup(pin, GPIO.IN, pull_up_down=GPIO.PUD_UP)

try:
    print("Monitoring buttons. Press Ctrl+C to stop.")
    while True:
        for pin, name in BUTTON_PINS.items():
            if GPIO.input(pin) == GPIO.LOW:  # Button pressed
                print(f"{name} pressed!")  # Optional: print to console
                time.sleep(0.3)  # Debounce delay
        time.sleep(0.1)  # Small delay to prevent high CPU usage

except KeyboardInterrupt:
    print("\nExiting program.")

finally:
    GPIO.cleanup()  # Cleanup GPIO on exit
