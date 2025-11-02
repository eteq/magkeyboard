from matplotlib import pyplot as plt
from matplotlib.patches import Rectangle
from PIL import Image

MM_TO_INCHES = 1 / 25.4

# all distances in mm

key_spacing = 19
key_size = 16

figx = 235
figy = 93


fontsize = 12
line_width = 1
dpi = 600

split = 93


def make_rect(lowerleft, ax, text=None, color='black', circle_colors=None):
    rect = Rectangle(lowerleft, key_size, key_size, facecolor='none', edgecolor=color, lw=line_width)
    ax.add_patch(rect)
    if text is not None:
        text = ax.text(lowerleft[0] + key_size / 2, lowerleft[1] + key_size / 2, text,
                          ha='center', va='center', fontsize=fontsize, color=color)
        
    if circle_colors is not None:
        n = len(circle_colors)
        for i, c in enumerate(circle_colors):
            circle = plt.Circle((lowerleft[0] + key_size * (i + 1) / (n + 1),
                                 lowerleft[1] + key_size / 2),
                                key_size / 6, color=c)
            ax.add_patch(circle)

    return rect, text

fig = plt.figure(figsize=(figx*MM_TO_INCHES, figy*MM_TO_INCHES), dpi=dpi)
fig.subplots_adjust(left=0, right=1, top=1, bottom=0)
ax = fig.subplots(1, 1)


fig2 = plt.figure(figsize=(figx*MM_TO_INCHES, figy*MM_TO_INCHES), dpi=dpi)
fig2.subplots_adjust(left=0, right=1, top=1, bottom=0)
ax2 = fig2.subplots(1, 1)

rows = [('QWERTYUIOPb', 0),
        ("tASDFGHJKL;'", key_spacing / 4 - key_spacing),
        ("sZXCVBNM,./s", key_spacing * 3 / 4 - key_spacing),
        ("ca 12 45 ac", key_spacing * 1 / 2 - key_spacing),
        ("   p3e6   ", key_spacing * 1 / 4),
       ]

lines_to_save = []
minx = maxx = miny = maxy = countl = countr = 0
for i, (row, offset) in enumerate(rows):
    for j, key in enumerate(row):
        xll = j*key_spacing + offset
        yll = i*-key_spacing

        color = 'black'
        if split is not None:
            if xll < split:
                color = 'blue'
                countl += 1
            else:
                color = 'red'
                countr += 1
        else:
            countl += 1
        if key != ' ':
            rect, _ = make_rect((xll, yll), 
                                ax=ax, text=key, color=color)
            minx = min(*rect.get_bbox().corners()[..., 0], minx)
            maxx = max(*rect.get_bbox().corners()[..., 0], maxx)
            miny = min(*rect.get_bbox().corners()[..., 1], miny)
            maxy = max(*rect.get_bbox().corners()[..., 1], maxy)

        fidx = float((i*len(row)+j)/len(rows)/len(row))
        cs = 2*fidx if fidx<=.5 else 1.
        ncs = 2*(fidx-.5) if fidx>=.5 else 0.

        if color =='blue':
            ccolors = [(cs ,cs , ncs), (ncs, cs, cs), (cs, ncs, cs)]
        else:
            ccolors = [(cs, ncs , ncs), (ncs, cs, ncs), (ncs, ncs, cs)]

        make_rect((xll, yll), ax=ax2, text='', color=color, 
                  circle_colors=ccolors)


        sidestr = ' L ' if xll < split else ' R ' if split is not None else ''
        lines_to_save.append(f"{key}{sidestr}{xll+key_size/2} {yll+key_size/2}")

for axi in [ax, ax2]:
    axi.set_xlim(minx, minx + figx)
    axi.set_ylim(miny, miny + figy)
    axi.set_aspect('equal')

    if split is not None:
        axi.axvline(split, c='k', ls=':', alpha=.25)


print('counts', countl, countr)

fig.savefig("keyboard_layout.png", dpi=dpi)

fig2.savefig("color_tests.png", dpi=dpi)

#inverted version
img = Image.open("color_tests.png")
newimg = img.point(lambda p: 255 - p)
newimg.putalpha(img.split()[-1])
newimg.save("color_tests_inverted.png")

with open("key_spec", "w") as f:
    for l in lines_to_save:
        f.write(l)
        f.write("\n")
