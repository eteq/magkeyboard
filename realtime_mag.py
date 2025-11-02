# meant to be used with circuitpython_testing.py on the keyboard

import serial
from matplotlib import pyplot as plt
import numpy as np


voltages = np.zeros((100, 25))

fig, ax = plt.subplots()
lines = ax.plot(voltages[:, 0], voltages)
print(len(lines))
ax.set_xlim(0, 100)
ax.set_ylim(0, 3.5)

def update_plot(voltages, i):
    x = np.arange(voltages.shape[0]) + i
    for line, voltage in zip(lines, voltages.T):
        line.set_xdata(x)
        line.set_ydata(voltage)
    ax.set_xlim(x[0], x[-1])
    fig.canvas.draw_idle()
    fig.canvas.flush_events()
    plt.pause(0.01)


s = serial.Serial('/dev/ttyACM0', 115200)

i = 0
for line in s:
    elems = line.strip().split(b',')
    if b'BLE:' in line:
        continue
    if len(elems) == 25:
        try:
            vs = [float(e) for e in elems]
        except ValueError:
            print('invalid line:', line)
            continue
        i += 1
    else:
        print('invalid line:', line)
        continue

    if i < 100:
        voltages[i] = vs
    else:
        voltages = np.roll(voltages, -1, axis=0)
        voltages[-1] = vs

        update_plot(voltages, i)
