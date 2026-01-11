# cat /dev/ttyACM0 > filename.dat
# run this script on filename.dat

from pathlib import Path
import argparse


import matplotlib.pyplot as plt
import numpy as np

parser = argparse.ArgumentParser()
parser.add_argument("input_file", type=Path)
parser.add_argument("-m", "--mean-subtract", action='store_true')
parser.add_argument("-o", "--outliers-sig", default=0, type=float)
parser.add_argument("-c", "--channels", default='', type=str)
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

v = np.array(vals)
if args.mean_subtract:
    v = v - v.mean(axis=0)

if args.channels:
    if args.outliers_sig != 0:
        raise ValueError('cannot set both outliers and channels')

    msk = np.zeros(v.shape[1],dtype=bool)
    for chnum in args.channels.split(','):
        msk[int(chnum.strip())] = True

    v = v[:,msk]

if args.outliers_sig != 0:
    stds = v.std(axis=0)
    stdall = v.std()
    msk = stds > args.outliers_sig*stdall
    print('selected channels', np.where(msk)[0], stds[msk])

    v = v[:,msk]


plt.plot(np.array(timestamps) - timestamps[0], v)
plt.show()
