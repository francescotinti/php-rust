#!/usr/bin/env python3
# gen-coda-guardia-s193.py — genera s193-coda-guardia.sh come COPIA DICHIARATA di s193-lancio-notte.sh
# (manifest s193-coda-guardia-copia.diff). Generato con assistenza AI (Claude Fable 5.2).
# Motivo: la guardia b5 della notte del 20/11 e' caduta sul gate di quiescenza (rc=8, nessuna misura) per una
# sonda inline dell'assistente col nome del binario nell'argv; la coda ripete la SOLA guardia, tag b5b, dopo la catena.
import os, sys
H = os.path.dirname(os.path.abspath(__file__))
src = open(os.path.join(H, "s193-lancio-notte.sh"), encoding="utf-8").read()
lines = src.split("\n")
i0 = lines.index("set -u")
cut = [k for k, l in enumerate(lines) if l.startswith("SPD=/private/tmp/")]
assert len(cut) == 1, cut
body = "\n".join(lines[i0:cut[0]])
def sub(old, new, n=1):
    global body
    assert body.count(old) == n, (old, body.count(old))
    body = body.replace(old, new)
sub('LOG="$H/catena-out/lancio-catena.log"; mkdir -p "$H/catena-out"',
    'LOG="$H/catena-out/coda-guardia.log"; mkdir -p "$H/catena-out"\n'
    'echo "$(date \'+%F %T\') attesa fine catena (catena-s193.done presente e lock assente)" >> "$LOG"\n'
    'while [ ! -e "$H/catena-out/catena-s193.done" ] || [ -e /private/tmp/phpr-measure.lock ]; do sleep 60; done\n'
    '[ -e "$H/../wp186-harness/s186-guardia-verdetto-b5b.out" ] && { echo "$(date \'+%F %T\') verdetto b5b ESISTE: esco" >> "$LOG"; exit 7; }')
sub('"s193 notte b5+t29+server-t4+t27 @ pin s185 ', '"s193 coda guardia b5b @ pin s185 ')
sub('WD="$H/catena-out/watchdog-catena.txt"; PD="$H/catena-out/catena-s193.done"',
    'WD="$H/catena-out/watchdog-coda.txt"; PD="$H/catena-out/coda-guardia.done"')
sub('quiet-decl-catena.txt', 'quiet-decl-coda.txt', 2)
sub('(b5, t29, server t4, t27)', '(coda: guardia b5b)')
sub(' — (1) guardia b5, (2) t29, (3) server t4, (4) t27" >> "$LOG"', ' — guardia b5b" >> "$LOG"')
sub('DONE="$H/catena-out/catena-s193.done"; rm -f "$DONE"', 'DONE="$H/catena-out/coda-guardia.done"; rm -f "$DONE"')
sub('s186-guardia-verdetto-b5.out', 's186-guardia-verdetto-b5b.out', 2)
sub('verdetto b5 ESISTE', 'verdetto b5b ESISTE')
sub('"rc=7 verdetto-b5"', '"rc=7 verdetto-b5b"')
sub('(1) GUARDIA b5 (A=', 'GUARDIA b5b (A=')
sub('RE3=3 TAG=b5 /bin/bash', 'RE3=3 TAG=b5b /bin/bash')
sub('guardia-out-b5/guardia.done', 'guardia-out-b5b/guardia.done')
# la riga di log dopo la guardia annuncia il passo (2): si tronca e si chiude
k = body.rindex('echo "$(date \'+%F %T\') guardia b5 fine rc=$rc1')
body = body[:k] + 'fine "coda fine: guardia b5b rc=$rc1 ($GR); lock rimosso" "rc=$rc1 guardia_b5b=[$GR]" "$rc1"\n'
for bad in ('TAG=b5 ', 'guardia-out-b5/', 'lancio-catena.log', 't29', 't27', 'catena-fetta1'):
    assert bad not in body, bad
head = """#!/bin/bash
# s193-coda-guardia.sh — CODA della notte di S-193 (emenda p.8 di s193-criterio-notte.md): ripete la SOLA guardia di costo,
# tag b5b, DOPO la fine della catena (catena-s193.done presente, lock assente), con lock proprio e gli STESSI gate di finestra.
# COPIA DICHIARATA di s193-lancio-notte.sh (manifest s193-coda-guardia-copia.diff, generatore gen-coda-guardia-s193.py)
# coi SOLI adattamenti: attesa della fine catena, log/done/watchdog/dichiarazione propri, tag b5b, tolti i passi 2-4.
"""
out = os.path.join(H, "s193-coda-guardia.sh")
open(out, "w", encoding="utf-8").write(head + body)
os.chmod(out, 0o755)
print("ok", len((head + body).split("\n")), "righe")
