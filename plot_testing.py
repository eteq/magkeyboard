# cat /dev/ttyACM0 > filename.dat
# run this script on filename.dat

from pathlib import Path
import argparse


import matplotlib.pyplot as plt
import numpy as np

parser = argparse.ArgumentParser()
parser.add_argument("input_file", type=Path)
args = parser.parse_args()

timestamps = []
vals = []
with args.input_file.open("r") as f:
    for line in f:
        spl = line.strip().split(',')
        if len(spl) != 25:
            continue

        try:
            ts = float(spl[0])
        except ValueError:
            continue

        timestamps.append(ts)
        lst = []
        for s in spl[1:]:
            lst.append(float(s))
        vals.append(lst)

plt.plot(np.array(timestamps) - timestamps[0], np.array(vals))
plt.show()