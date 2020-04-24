#! /usr/bin/python3

import os
import os.path
import sys


_dirname = os.path.dirname(os.path.abspath(__file__))

if not os.path.exists(os.path.join(_dirname, 'corpus')):
    os.mkdir(os.path.join(_dirname, 'corpus'))

for root,dirs,files in os.walk(os.path.join(_dirname, '../resources/')):
    for name in files:
        if not name.endswith('.ttf'):
            continue
        # move this into corpus
        fname = os.path.join(root,name)
        dest = os.path.join(_dirname, 'corpus', name)
        with open(fname, mode='rb') as fin:
            b = fin.read()
            if len(b) + 4 < 1024 * 1024:
                with open(dest + ".1", mode='wb') as fout:
                    fout.write(b + b'\x00\x00\x00g')
                with open(dest + ".2", mode='wb') as fout:
                    fout.write(b + b'\x00\xe3\x92\xa8')
                with open(dest + ".3", mode='wb') as fout:
                    fout.write(b + b'\x00\xe2\x88\x91')

