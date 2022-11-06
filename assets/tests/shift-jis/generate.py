import sys

def generate(version, e):
    ok = set()
    spec = open("../../build/shift-jis.txt", "r").readlines()

    for line in spec:
        if version < 1983:
            if "[1983]" in line:
                continue
        if version < 1990:
            if "[1990]" in line:
                continue
        if version < 2000:
            if "[2000]" in line:
                continue
        if version < 2004:
            if "[2004]" in line:
                continue

        if "<reserved>" in line:
            continue
        if line.startswith("#"):
            continue
        if line.startswith("0x"):
            code = line.split()[0]
            ok.add(int(code, 16))

    a = bytearray()
    b1 = bytearray()
    b2 = bytearray()
    for i in range(0x00, 0x80):
        if i in ok:
            a.extend(f"{i:02x}: ".encode("ascii"))
            a.append(i)
            a.extend('\n'.encode("ascii"))
        else:
            b1.append(i)

    for i in range(0xA1, 0xE0):
        if i in ok:
            a.extend(f"{i:02x}: ".encode("ascii"))
            a.append(i)
            a.extend('\n'.encode("ascii"))
        else:
            b1.append(i)

    for i in range(0x80, 0xA0):
        for j in range(0x40, 0xFE):
            if (i << 8) | j in ok:
                a.extend(f"{i:02x}{j:02x}: ".encode("ascii"))
                a.append(i)
                a.append(j)
                a.extend('\n'.encode("ascii"))
            else:
                b2.append(i)
                b2.append(j)

    for i in range(0xE0, 0xFE):
        for j in range(0x40, 0xFE):
            if (i << 8) | j in ok:
                a.extend(f"{i:02x}{j:02x}: ".encode("ascii"))
                a.append(i)
                a.append(j)
                a.extend('\n'.encode("ascii"))
            else:
                b2.append(i)
                b2.append(j)


    with open(f"{version}.ok.shift-jis.txt", mode="bw") as f:
        f.write(a)

    if len(b1) > 0:
        with open(f"{version}.error.one-byte.shift-jis.txt", mode="bw") as f:
            f.write(b1)

    if len(b2) > 0:
        with open(f"{version}.error.two-byte.shift-jis.txt", mode="bw") as f:
            f.write(b2)

    with open(f"{version}.ok.utf-8.txt", mode="bw") as f:
        s = a.decode(e)

        # expected 815c -> U+2014 'EM DASH'
        # got              U+2015 'HORIZONTAL BAR'
        s = s.replace("\u2015", "\u2014")

        # expected 81d4 -> U+ff5f 'FULLWIDTH LEFT WHITE PARENTHESIS'
        # got              U+2985 'LEFT WHITE PARENTHESIS'
        s = s.replace("\u2985", "\uff5f")

        # expected 81d5 -> U+ff60 'FULLWIDTH RIGHT WHITE PARENTHESIS'
        # got              U+2986 'RIGHT WHITE PARENTHESIS'
        s = s.replace("\u2986", "\uff60")

        f.write(s.encode('utf-8'))
    

generate(1997, "shift_jis_2004")
generate(2004, "shift_jis_2004")
