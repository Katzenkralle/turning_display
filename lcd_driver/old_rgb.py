from random import randrange
from unittest import case
from rpi_ws281x import PixelStrip, Color

bright = None
#Init Lightstrip
LED_COUNT = 72        # Number of LED pixels.
LED_PIN = 10        # GPIO pin connected to the pixels (10 uses SPI /dev/spidev0.0).
LED_FREQ_HZ = 800000  # LED signal frequency in hertz (usually 800khz)
LED_DMA = 10          # DMA channel to use for generating signal (try 10)
LED_BRIGHTNESS = 255  # Set to 0 for darkest and 255 for brightest
LED_INVERT = False    # True to invert the signal (when using NPN transistor level shift)
LED_CHANNEL = 0       # set to '1' for GPIOs 13, 19, 41, 45 or 53
strip = PixelStrip(LED_COUNT, LED_PIN, LED_FREQ_HZ, LED_DMA, LED_INVERT, LED_BRIGHTNESS, LED_CHANNEL)
strip.begin()



def color(i):
    switcher = {
        0: Color(255, 0, 0),
        1: Color(255, 50, 0),
        2: Color(255, 100, 0),
        3: Color(0, 255, 0),
        4: Color(0, 0, 255),
        5: Color(100, 0, 90),
        6: Color(200, 0, 100),
        7: Color(0, 0, 0)
    }
    return(switcher.get(i))

def wheel(pos):
    """Generate rainbow colors across 0-255 positions."""
    if pos < 85:
        return Color(pos * 3, 255 - pos * 3, 0)
    elif pos < 170:
        pos -= 85
        return Color(255 - pos * 3, 0, pos * 3)
    else:
        pos -= 170
        return Color(0, pos * 3, 255 - pos * 3)

def set_color(R,G,B, bright, start_pixel, end_pixl):
    strip.setBrightness(bright)
    for i in range(end_pixl - start_pixel):
        strip.setPixelColor(i + start_pixel, Color(R,G,B))
        strip.show()
    return()

def rotate(bright, iterations):
        for j in range(256 * iterations):
            for i in range(strip.numPixels()):
                strip.setPixelColor(i, wheel(
                    (int(i * 256 / strip.numPixels()) + j) & 255))
            strip.setBrightness(bright)
            strip.show()
        return()

def paint(pixel_list, colore_list, bright):
    strip.setBrightness(bright)
    for l in range(len(pixel_list)):
        strip.setPixelColor(pixel_list[l], colore_list[l])
        strip.show() 
    return()

def puls(R,G,B, rand_min, rand_max, bright_min, bright_max):
    global bright
    rand = randrange(rand_min, rand_max)
    plus_minus = randrange(0,2)
    if bright == None:
        bright = ((bright_min + bright_max)/2)
    if plus_minus == 0 and bright < bright_max:
        bright += rand
    elif plus_minus == 1 and bright > bright_min:
        bright -= rand
    if bright < bright_min:
        bright = bright_min
    elif bright > bright_max:
        bright = bright_max
    strip.setBrightness(int(bright))
    strip.show()
    for i in range(72):
        strip.setPixelColor(i, Color(R,G,B))
        strip.show()
    return()
