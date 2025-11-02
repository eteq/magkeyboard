# note that make_keyboard_layout.py must be run first to generate the layout

# parameters
pcbside = 'L'  # 'L' or 'R'
unused_pile = (225, 100)

#imports
from kipy import KiCad
from kipy.geometry import Vector2


#the actual script
kicad = KiCad()
board = kicad.get_board()

keyxys = []
with open("key_spec") as f:
    for l in f:
        ls = l.strip()
        if ls == '':
            continue
        name, side, x, y = ls.split()
        if side == pcbside:
            keyxys.append((float(x), float(y)))

# start by identifying the starting points in the PCB:
# assumptions:
# * K00 is to be treated as the upper-left-most key, and everything else is relative to that
# * the X00 components are the "template" - that is, their offsets relative to K00 and orientations are to be replicated across the key grid.

commit = board.begin_commit()

footprints = board.get_footprints()
name_to_fp = {fp.reference_field.text.value: fp for fp in footprints}

assert len(set(name_to_fp.keys())) == len(name_to_fp), "duplicate footprint names, cannot continue"


offset_origin = name_to_fp['K00'].position
fpspec = {name: (fp.position - offset_origin, fp.orientation) 
          for name, fp in name_to_fp.items()
          if name in ['K00', 'D00', 'S00', 'C001', 'C002']}

Ks = sorted([nm for nm in name_to_fp if len(nm)==3 and nm[0]=='K'])
unplaced = set(Ks)
toplace = []

x0 = y0 = None
for Ki, (x, y) in zip(Ks, keyxys):
    if Ki == 'K00':
        # this should always be first, but if not, the subtraction below will fail
        x0 = x
        y0 = y
    else:
        thiskey_code = Ki[1:]

        key_center = Vector2.from_xy_mm(x-x0, y0-y) + offset_origin
        print(f"Placing {thiskey_code} at {key_center}")

        for nmpspec, (off, ori) in fpspec.items():
            target = nmpspec.replace('00', thiskey_code)
            name_to_fp[target].position = key_center + off
            name_to_fp[target].orientation = ori

            toplace.append(name_to_fp[target])

    unplaced.remove(Ki)

print('these keys were *not* placed:', unplaced, 'sending them to the pile')

for Ki in unplaced:
        thiskey_code = Ki[1:]

        for nmpspec, (off, ori) in fpspec.items():
            target = nmpspec.replace('00', thiskey_code)
            name_to_fp[target].position = Vector2.from_xy_mm(*unused_pile)

            toplace.append(name_to_fp[target])


board.update_items(toplace)
board.push_commit(commit)

