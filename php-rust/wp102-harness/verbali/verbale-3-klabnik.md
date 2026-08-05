# Verbale sedia 3 — Klabnik (spec, testabilità, matrici e gate) — Concilio WP-102

## VERDETTO

S-100 regge nella sostanza (flip eseguito con gate reali, contratto value-parsed,
denti derivati dal contratto). Ma UNA refutazione è capitale: la motivazione
dichiarata del gate carry-over è FALSA a livello macchina. Tre refutazioni
minori su carve-out, matrice denti e bozza S-101.

## Refutazioni

**R1 (CAPITALE) — «il flip cambia solo la costante» è falso.** Il diff
per-test (A-KL-101-4, evidence 20:41) è stato eseguito sul binario candidato
a2772e62 (albero 9d0d001), non sul pin. La motivazione dichiarata cade sul
`git show fb861e4 --stat`: il flip tocca `compile/func.rs` (compile_body legge
`ctx.reg_lower`), `compile/mod.rs` (+29: `compile_program_with_mode`, entry) e
150 righe di `reg_lower.rs` — è un ricablaggio dell'entry di compilazione, non
una costante. Mitigante reale: sul pin girano corpus 1418 per NOME ×2 modi +
batteria 1735/0 + server bimodale. Buco residuo: il diff BYTE dei chunk FAIL
non è mai stato ri-giudicato sul pin — classe carry-over (WP-94).

**R2 — la carve-out nondet è provata a metà ed esenta troppo.** L'entropia
intra-modo è provata SOLO flag-off (8bdbc200≠70cfe091); la gamba on è
presunta. E l'esenzione è chunk-wide: una divergenza off↔on VERA nel resto
del chunk dei 3 settype (testo warning, ordine) passerebbe sotto il tappeto.
Il «quarto test urla» solo se l'entropia si manifesta nella singola coppia di
run (R=1 per modo): rilevazione probabilistica, non garantita.

**R3 — post-flip il percorso di produzione (flag ASSENTE) è giudicato da UN
bit.** Corpus, funnel e trappole girano tutti con valori ESPLICITI; il
braccio absent di antiputenv verifica solo `any(REG_FORMS)==DEFAULT_ON`.
Nessun dente asserisce absent ≡ `=1` a parità di DUMP. Il tappeto-tautologia
è però evitato: `mode_contract_default_is_on_post_flip` (reg_lower.rs:990)
pinna il letterale. Ma antiputenv.rs:108 cita ancora il dente MORTO
`mode_contract_default_is_off_pre_flip`: un commento che indica un tripwire
inesistente è un gate di carta.

**R4 — bozza S-101, punto 3 refutato.** «L'emissione non cambia ⇒ batteria +
corpus bastano» confonde parità d'emissione con parità di RUNTIME: H-C1
(clone→prestito) cambia il ciclo di vita Zval — timing dei `__destruct`,
refcount osservabili, aliasing — la classe di bug che SOLO il collaudo WP ha
storicamente trovato (leak WP-78). Inoltre la matrice fixture (hook, __get,
ref, readonly, visibilità) è aperta: mancano per NOME `&$o->p` (alias byref),
timing distruttori, weakref/GC, coercizione typed. E nessuna banda numerica è
pre-registrata per la ri-baseline (punto 1).

## Emendamenti

- **A-KL-102-1**: ri-eseguire `s100-corpus-diff.sh` sul PIN (le due passate
  costano; la definizione operativa esiste già). L'evidenza di un gate nasce
  dall'albero giudicato.
- **A-KL-102-2**: carve-out → normalizzazione per TOKEN (strip del solo hex
  `random_bytes`, byte-diff del resto del chunk) + prova d'entropia
  intra-modo ANCHE flag-on.
- **A-KL-102-3**: dente absent ≡ `=1` a parità di dump-hash; sanare il
  commento morto in antiputenv.rs:108.
- **A-KL-102-4**: matrice fixture H-C1 chiusa per NOME PRIMA del codice,
  inclusi alias byref, timing `__destruct`, weakref/GC, typed coercion.

## KS (criteri vincolanti)

- **KS-KL-102-1**: nessun gate futuro cita evidenza prodotta su un albero
  diverso dal promosso senza allegare il diff macchina dei crates runtime.
- **KS-KL-102-2**: H-C1 si iscrive col SOFFITTO pre-registrato: il profilo dà
  clone 7,6% + deref 2,1% (+ quota drop/gc_note) ⇒ guadagno max ~27% ⇒ prop
  ≥ ~9× anche a successo pieno. H-C1 NON è la cura del 12,4→3: dichiararlo.
- **KS-KL-102-3**: WP pair di parità NON derogabile per H-C1 (cambia il
  runtime, non l'emissione); rilevazione nondet con R=2 intra-modo sul
  fail-set.
