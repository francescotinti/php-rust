# VETI — i veti più vecchi di 3 sessioni (REGOLE §8, delibera S-186): escono dal NEXT e restano qui per NOME

Generato con assistenza AI (Claude Fable 5). Il NEXT tiene solo i veti delle ultime 3 sessioni; questa lista non si potà.

- rc di gate da pipe (`rc` dal comando, mai da `| tee`)
- REPORT_GAP cumulativo (solo se la sessione ha misurato full/media)
- wipe naive di uploads (solo guardia backup/wipe/restore)
- stash senza smoke (collaudo-nell'atto: `scripts/pin-phpr.sh` / `pin-server.sh`)
- build, CI e misura concorrenti (S-184: swap 15G, Data 2G, phpt-runner OOM)
- daemon lanciato con tree in checkout detached (S-184 #1)
- pgrep auto-match del monitor inline (sonde e monitor in un FILE, S-184 #2)
- tolleranze di gate senza base (intervallo MOTIVATO; se morde si rifà la build, non la soglia — S-184 #3)
- `rm` non verificato (dopo rm/patch: `ls`/`grep`, il rc non basta — S-184 #4)
- A/B a cifre assolute tra binari diversi (solo direzione; stesso binario per le bande — REGOLE §3)
- leve su `run_loop` senza disasm come gate (S-104; Δsp_refs ≤ +97 vs pin s182, S-184)
- giudice che scrive rc=0 con `.time` mancanti (S-184 bis #5: exit 7 = verdetto non emesso)
- watchdog che campiona solo il wrapper `time` (S-184 bis #6: sample+kill di tutti i discendenti)
- `$this`/`$nome` PHP in doppi apici sotto `set -u` (S-185: `\$this` + assert nel generatore)
- mutante senza prova sui binari PRIMA del lancio (S-185; S-186 #5: attribuzione xctrace a gate della build)
- attesa «nessuna nuova chiamata nel loop» senza contare la chiamata della leva stessa (S-185)
