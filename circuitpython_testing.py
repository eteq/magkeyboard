import time
import pwmio
import busio
import board
import digitalio
import analogio
from microcontroller import pin

from neopixel import NeoPixel

from adafruit_lsm6ds.lsm6ds3 import LSM6DS3

TEST_COLORS = [(1,0,0), (0,1,0), (0,0,1), (1,1,0), (0,1,1), (1,0,1), (1,0.5,0), (1,0,0.5), (0.5,0.5,0.5)]
KEY_BRIGHTNESS = 10
COLOR_TIME = 0.15

imu_pwr = digitalio.DigitalInOut(board.IMU_PWR)
imu_pwr.switch_to_output(True)
time.sleep(0.04) #datasheet says 35ms startup time
imu_i2c = busio.I2C(board.IMU_SCL, board.IMU_SDA)
imu = LSM6DS3(imu_i2c)

print("imu initial accel", imu.acceleration, 'gyro', imu.gyro)

def set_led_frac(led, frac):
    led.duty_cycle = int(65535 * (1-frac))

red = pwmio.PWMOut(board.LED_RED)
green = pwmio.PWMOut(board.LED_GREEN)
blue = pwmio.PWMOut(board.LED_BLUE)

def set_rgb_led(r,g,b):
    set_led_frac(red, r)
    set_led_frac(green, g)
    set_led_frac(blue, b)

print('cycling led colors')
for r,g,b in TEST_COLORS:
    set_rgb_led(r, g, b)
    time.sleep(COLOR_TIME)

vhi = digitalio.DigitalInOut(board.D6)
vhi.switch_to_output(True)  # turns *on* VHI

key_npx = NeoPixel(pin.P0_15, 24)

print('cycling key colors')
for r,g,b in TEST_COLORS:
    key_npx.fill((int(KEY_BRIGHTNESS*r), int(KEY_BRIGHTNESS*g), int(KEY_BRIGHTNESS*b)))
    time.sleep(COLOR_TIME)

print("counting up keys")
for i in range(24):
    key_npx.fill((0,0,0))
    r,g,b = TEST_COLORS[i % len(TEST_COLORS)]
    key_npx[i] = (int(KEY_BRIGHTNESS*r), int(KEY_BRIGHTNESS*g), int(KEY_BRIGHTNESS*b))
    time.sleep(COLOR_TIME/2)

print('testing vhi power - lights should come on white, off, then back on, then off.  Single solid on means not working')
key_npx.fill((KEY_BRIGHTNESS, KEY_BRIGHTNESS, KEY_BRIGHTNESS))
time.sleep(0.5)
vhi.value = False  # lights should turn off
time.sleep(0.5)
vhi.value = True  # lights should turn back on after a fill
key_npx.fill((KEY_BRIGHTNESS, KEY_BRIGHTNESS, KEY_BRIGHTNESS))
time.sleep(0.5)
vhi.value = False  # turn off power just for savings

amux = digitalio.DigitalInOut(board.NFC1)
bmux = digitalio.DigitalInOut(board.NFC2)
amux.switch_to_output(False)
bmux.switch_to_output(False)

# just turn all the muxes on
muxen01 = digitalio.DigitalInOut(board.D8)
muxen01.switch_to_output(False)

muxen23 = digitalio.DigitalInOut(board.D9)
muxen23.switch_to_output(False)

muxen45 = digitalio.DigitalInOut(board.D10)
muxen45.switch_to_output(False)

adc0 = analogio.AnalogIn(board.A0)
adc1 = analogio.AnalogIn(board.A1)
adc2 = analogio.AnalogIn(board.A2)
adc3 = analogio.AnalogIn(board.A3)
adc4 = analogio.AnalogIn(board.A4)
adc5 = analogio.AnalogIn(board.A5)
# note we change the order because the second two ADC channel sets are swapped
adcs = (adc0, adc1, adc4, adc5, adc2, adc3)

adc_scale = adc0.reference_voltage * 2**-16

vhi.value = True # allow the pixels to light up
values = [0]*24
while True:
    key_npx.fill((0,0,0))

    amux.value = False
    bmux.value = False
    #0
    for i, adc in enumerate(adcs):
        values[i*4] = adc_scale*adc.value

    amux.value = True
    #1
    for i, adc in enumerate(adcs):
        values[i*4+1] = adc_scale*adc.value

    bmux.value = True
    amux.value = False
    #2
    for i, adc in enumerate(adcs):
        values[i*4+2] = adc_scale*adc.value

    amux.value = True
    #3
    for i, adc in enumerate(adcs):
        values[i*4+3] = adc_scale*adc.value

    print(time.monotonic(), ', ' + str(values)[1:-1])

    for i, v in enumerate(values):
        diff = abs(v - 1.65)
        if diff > 0.5:
            ndiff = diff / 1.65
            h = ndiff * 6
            x = abs(h% 2 - 1)
            if h < 1:
                r, g, b = 1, x, 0
            elif h < 2:
                r, g, b = x, 1, 0
            elif h < 3:
                r, g, b = 0, 1, x
            elif h < 4:
                r, g, b = 0, x, 1
            elif h < 5:
                r, g, b = x, 0, 1
            else:
                r, g, b = 1, 0, x
            key_npx[i] = (int(KEY_BRIGHTNESS*r), int(KEY_BRIGHTNESS*g), int(KEY_BRIGHTNESS*b))

# >>> board.
# A0              A1              A2              A3
# A4              A5              CHARGE_RATE     CHARGE_STATUS
# D0              D1              D10             D2
# D3              D4              D5              D6
# D7              D8              D9              I2C
# IMU_INT1        IMU_PWR         IMU_SCL         IMU_SDA
# LED             LED_BLUE        LED_GREEN       LED_RED
# MIC_PWR         MISO            MOSI            NFC1
# NFC2            PDM_CLK         PDM_DATA        READ_BATT_ENABLE
# RX              SCK             SCL             SDA
# SPI             TX              UART            VBATT
# __dict__        board_id
