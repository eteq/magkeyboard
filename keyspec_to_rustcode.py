key = []
side = []
x = []
y = []

with open('key_spec') as f:
    for l in f:
        ls = l.strip().split()
        if len(ls) == 3:
            ls.insert(0, '')
        if len(ls) == 4:
            key.append(ls[0])
            side.append(ls[1])
            x.append(float(ls[2]))
            y.append(float(ls[3]))

for s in side:
    assert s in ['L', 'R'], f"Side must be L or R, got {s}"


lasty = y[0]
lastx = x[0]
for xi, yi in zip(x, y):
    if yi == lasty:
        if xi < lastx:
            raise ValueError("X coordinates must be increasing left to right in a row")
    else:
        if yi > lasty:
            raise ValueError("Y coordinates must be decreasing top to bottom in rows")
        lasty = yi

indent = (' '*4)*1

for sidei in ['L', 'R']:
    print(f"for side {sidei}:")
    i1 = 0
    i2 = 0
    for k, s in zip(key, side):
        keyname = f'{i1}{i2}'
        if k.isupper():
            k1 = k.upper()
            k2 = k.lower()
        else:
            k1 = k
            k2 = '<FIX>'

        if s == sidei and k != '':
            print(f'{indent}m.insert(({keyname}, Layer::Default), KeyboardUsage::Keyboard{k1}{k2}).expect("no space for key!");')

            if i2 >= 3:
                i1 += 1
                i2 = 0
            else:
                i2 += 1