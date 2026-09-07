# Revisione S-172 — lente PROCESSO (revisore adversariale)

**VERDETTO: REGGE CON RILIEVI** — pre-registrazione vera (criterio 397f8f78 23:19:27, P1 23:26:17, P2 23:27:40, misura 23:35:59-23:41:12; attese [8;20]/[12;30] già in `s172-ab-leva.sh:85`, mtime 23:20:31; nessun job CI nella finestra). La D nominata su prop-dq regge; non reggono l'ATTRIBUZIONE a P2 e il presidio semantico.

1. **Cifra composta tra binari** (REGOLE §3-4, §8): `s172-verdetto.out` r.3-4 scrive «P2 = −4,7» e «B−C=+4.67 coerente con…» (prosa, non «solo esiti»). Il criterio p.3 promette «banda dichiarata», p.4 non ne dichiara alcuna; `s172-ab-leva.sh:84` giudica solo «oltre il rumore». C è SEMPRE in 5ª/6ª posizione (`s172-ab-leva.sh:59`, mai alternato): la scelta C>B «per D maggiore» poggia su un contrasto non alternato.
2. **Copia-gate del build stantio**: `s172-leva-build-copia-v3.diff` (aba8f5bd 23:26) non contiene l'emenda touch (`s172-leva-build.sh:25-29`, 23:32:55). L'adattamento «archivio del commit», mai collaudato, è fallito alla prima corsa (C==B, `catena.log` 23:32:15) = «scoperto qualcosa di mai collaudato» (§2): va CONTATO, non «nessuno contato» (`s172-verdetto.out` r.10).
3. **PIN_REGISTRY errato**: bracci B/C registrati «sorgente @ 57b21f4b» / «@ 18fe36c9» (HEAD allo stash, `scripts/pin-phpr.sh:63`) invece di c419f29a/59ca87fb. Identità del pin a contenuto (5f2dff7d vs e396498b, 47 B UUID/firma): ereditata S-171, ok.
4. **Presidio fx-sl2 VUOTO per il probe P1**: `ic.get(sk)` è per-sito (`git show c419f29a`: `let Some((cid1,gslot)) = ic.get(sk) else break 'f false`) ⇒ le righe single-shot 20-64 di `fx-sl2.php` girano a IC FREDDA e non entrano nel ramo sigillato; solo i loop r.17/18/101 lo esercitano, su prop pubbliche da scope globale. Scoperte: readonly inizializzata in-scope alla 2ª scrittura (`in_place` salta `write_property_at`), `public float $f = 1` con slot Long, private via `$this`, `__set`, lazy, enum. Nessun mutante su P1/P2.
5. **Guardia cieca a P2**: arith-dq è BinarySCSCDst, non tocca `BinarySTDst`, che sta in OGNI `$s OP= expr` (str, arr, calls). Micro calls/str/arr/re rinviate alla promozione (in corso). Dente run.rs 7200 > 7091 non dichiarato (p.5).
6. **P1 senza corpo `#[cold]`** (bl +14 in B, `s172-verdetto.out` r.9): contraddice la testa di p.2 («corpo esatto fuori linea»); dichiarato solo a posteriori.

## Azioni S-173
1. fx-sl2 riscritta: ogni forma P1 in loop ≥2 iterazioni (IC calda) + readonly in-scope 2ª scrittura, `float $f = 1`, `$this` private/ereditata, `__set`, lazy, enum; mutante abortivo P1/P2 (copia di `s172-mutante-sl1.sh`).
2. Prima di accreditare P2: A/B alternato B↔C; nel verdetto solo direzione, nessuna cifra.
3. Rigenerare il manifest build, contare l'incidente, correggere PIN_REGISTRY (`--braccio` accetti il commit sorgente).
4. Promozione: dente 7200 dichiarato + micro R=5 su tutte le categorie PRIMA del pin s172.
