from time import sleep
import RPi.GPIO as GPIO

DIR = 20   # Direction GPIO Pin
STEP = 21  # Step GPIO Pin
CW = 1     # Clockwise Rotation
CCW = 0    # Counterclockwise Rotation
SPR = 48   # Steps per Revolution (360 / 7.5)
ENA = 26
GPIO.cleanup()
GPIO.setmode(GPIO.BCM)
GPIO.setup(DIR, GPIO.OUT)
GPIO.setup(STEP, GPIO.OUT)
GPIO.output(DIR, CW)
GPIO.setup(ENA, GPIO.OUT)

TOTAL_ROUND = 800

step_count = SPR
delay = .0005
GPIO.output(ENA, GPIO.HIGH)

while True:
    for i in range(TOTAL_ROUND):
        print(i)
        GPIO.output(STEP, GPIO.HIGH)
        sleep(delay)
        GPIO.output(STEP, GPIO.LOW)
        sleep(delay)

    sleep(delay*2)
GPIO.cleanup()