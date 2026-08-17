# S-150 — atto d'identità del candidato BT1 (emenda al criterio p.1, REGOLE §3: criterio emendato RIESEGUENDO la prova)

**Fatto**: la build ricetta canonica NON riproduce l'hash del braccio B
giudicato (`ac26375aa0e8fef0`, s149-ab-bt1-verdetto.out) ma dà
`cbbe71735effb165`. Meccanismo NOMINATO e provato al byte:

1. Sorgente INVARIATO: `git diff 6a7adc8..HEAD -- crates/ Cargo.toml
   Cargo.lock` VUOTO (il commit leva è 6a7adc8; dopo, solo doc/rotazione).
2. Il braccio B fu costruito in `CARGO_TARGET_DIR=/private/tmp/phpr-bt1-target`
   (task S-149 `buxxls66v`) **senza** `SOURCE_DATE_EPOCH=0`; il gemello
   giudicato è conservato (scratchpad S-149, `phpr-bt1-s149`, hash
   `ac26375aa0e8fef0`, 15.108.800 B).
3. Ri-build ricetta (CON `SOURCE_DATE_EPOCH=0`) nello STESSO target
   `/private/tmp/phpr-bt1-target` → `2dd3066d9bbb61dc`, 15.108.800 B.
   `cmp -l` vs gemello: **92 byte** in 4 range: 2105–2120 (LC_UUID),
   12289163–12289182 (stringhe `Aug 17 2026`/`00:39:31` ↔ `Jan  1
   1970`/`00:00:00` — l'unico `__DATE__/__TIME__` embedded), 2 pagine di
   firma ad-hoc (32+32 B). `diff strings`: SOLO le due stringhe data/ora.
4. Build canonica `cbbe71735effb165` vs `2dd3066d9bbb61dc` (stessa ricetta,
   target diverso): **47 byte** in 2 range: LC_UUID (16 B) + una pagina di
   firma (32 B). `diff strings`: ZERO. **Il contenuto (codice+dati) è
   byte-IDENTICO: il path del target non entra nel codice.**

**Conclusione (emenda p.1)**: candidato di promozione = build ricetta
canonica **`cbbe71735effb165`**; l'identità col braccio giudicato è provata
al byte modulo {timestamp cosmetico, LC_UUID, firma ad-hoc derivata}.
A rinforzo, la catena ri-giudica il candidato: fixture fx-backtrace byte-id
(già PASS sul canonico, gate collaudato ANCHE in negativo su s145 rc=1) +
conferma m-backtrace R=5 nelle guardie.

**Lezione**: un braccio costruito fuori ricetta (env parziale) produce un
hash NON riproducibile; il gemello va stashato SEMPRE (qui c'era — è ciò che
ha reso la prova possibile) e l'atto A/B dovrebbe registrare la RICETTA
ESATTA del braccio B, env incluso.
