# Concilio WP-103 — SINTESI DI CONVERGENZA (su S-101 e programma S-102)

## §FONDAMENTALI (prima di tutto, regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: Gregg (mandato inverso)
dà AMMESSA. Nuovo per NOME: **prop 12,4→11,5** con DUE leve promosse ciascuna
dal SUO criterio pre-registrato (H-C1a Δ=7,3 ns/iter in banda; H-C1b Δ=6,0 ≥
pavimento, sotto banda con refutazione della stima REGISTRATA); trasversali
arith 12,7→12,2 e calls 7,9→7,3; census che ARBITRA (P1 ✓, P2 6 coppie/iter,
P3 4 gc_note/iter, 2 correzioni allo statico); 50% run_loop decomposto
(21,2% dispatch + ~26,6% pila operandi, attribuzione DICHIARATA porosa);
coppia WP nei 2 modi senza regressioni di parità; batteria 1737/0.

**(b) Contatore sessioni-senza-misura**: full/media = **WP-101 = QUESTA
sessione (0)**; giudice: tutte e sei le categorie fresche pre→post nella
stessa sera.

**(c) Rischio d'oggetto più trascurato** (Gregg): il **21,2% di run_loop
resta SENZA NOME** (dispatch? bounds? fetch decode?) e il **rumore full-peak
della gamba PHPR non è mai stato misurato** (solo quello oracle): ogni banda
peak futura pende da lì.

## Verdetti di fase 1 (9/9: nessun MI OPPONGO alle promozioni; 5 capitali)

Verbali VINCOLANTI in `verbali/verbale-*.md`; note di team in
`verbali/team-*.md`. Refutazioni capitali:

1. **«Se si tocca il server» confonde codice server con runtime eseguito**
   (Pedersen): il runtime È GIÀ cambiato (gc_note+PropGet/PropSet girano per
   ogni richiesta) — il collaudo del pin php-server 2c4242b6 è debito NON
   condizionato, PRIMO ATTO S-102, minimo = sentinella estesa bimodale +
   mode-probe + dente capture-boundary (output da __destruct, ≥2 richieste
   stesso worker, byte-id). Adottata dal team catena: il collaudo BLOCCA
   (un pin non graduato a runtime cambiato inquina il registro).
2. **«alloc/iter≈0» non è provato dallo strumento usato** (Leijen): le
   stats mimalloc a pagine sono CIECHE al churn malloc/free bilanciato — la
   gamba alloc si rifà a mem-census diretto (global_allocator); la cifra
   S-101 resta come indizio, non come verdetto.
3. **La metà emissione del dente absent≡`=1` è `f(x)==f(x)`** (Hejlsberg
   R-HE-103-1): `compile_mode(mode_from_env(None))` vs
   `compile_mode(mode_from_env("1"))` compila due volte con lo stesso bool —
   copertura fabbricata; la forma vera è la coppia in SOTTOPROCESSO (env
   assente vs `=1`) giudicata dal dump-diff (A-HE-103-3).
4. **«Gli inlined sovracontano» refutata come LEGGE** (Gregg R-GR-103-1):
   n=1 e segno non garantito — la lezione si riscrive «l'attribuzione a
   campioni sui simboli inlined ha errore di FATTORE ~2 A SEGNO IGNOTO:
   il costo/evento fa fede solo dall'A/B».
5. **«strong_count non osservabile» è FALSO come detto** (Matsakis): nel VM
   esistono ≥6 osservatori a CONTEGGIO ASSOLUTO (`Rc::strong_count == n`);
   H-C1b resta sound (verificato da Hoare sul codice e coperto dai gate),
   ma l'INVARIANTE va nominata (INV-RECV-1) e auditata PRIMA di estendere
   il move ad altre forme (KS-MA-103-2: estensione senza audit = reject).

Non capitali che entrano nell'ordine: attribuzione «26,6% pila» POROSA
(Bak+Gregg: arbitro = census push/pop per sito-opcode/primitiva + leva-nulla
di taratura; VIETATO derivarne attesi finché non contato — KS congiunto);
punto 4 a modo FISSATO off/off (Leijen: l'effetto-modo −116 MiB > l'oggetto
+95; spread ≥48 MiB ⇒ bisect vietato); fixture famiglia MOVE incomplete per
NOME (Stogov: destruct-reenter-PropSet, typed-write-coercion, clone;
Matsakis: lazy-init-drop-ultima-ref; +gc-mid-arm); gate-igiene Klabnik
(fixture-set pinnato a 13 per NOME o VOID; expected-diff §3.13 a path
normalizzato; staging per FILE nominati); Hoare: predicato unico
`is_gc_container` (lista specie duplicata 3×, componibile con la guardia a
due livelli di Stogov), buco Generator PRE-esistente da provare/fixturare.

## Nuove candidate NOMINATE (con canale contato)

- **H-C2 «drop fast-out scalare»** (Bak): ~11 drop/iter di drop-glue su
  Long; banda [8,22] ns/iter con pavimento ½ (motivazione a segno ignoto,
  emendata da Gregg). Entra DOPO il census pila (stesso strumento).
- **Slot-diretti per operandi Prop-op / ricevitore da slot**: INAMMISSIBILE
  com'era in bozza (Matsakis: il controfattuale DOPPIO-CONTA ns di un
  meccanismo che il move ha già eliminato; Hoare: rischio collisione col
  divieto di borrow) — si riscrive SOLO dopo census pila, con dump-diff
  come primo giudice e meccanismo dichiarato (clone-dallo-slot o sigillo).

## Ordine DEFINITIVO S-102 (regola di ammissione applicata)

1. **Collaudo php-server 2c4242b6** (debito NON condizionato, Pedersen):
   sentinella estesa bimodale + mode-probe + dente capture-boundary; niente
   cifre server né nuove build prima del grado.
2. **Guardie della famiglia MOVE** (bloccano ogni estensione, non la
   promozione fatta): audit INV-RECV-1 dei ≥6 osservatori assoluti di
   strong_count (esito per NOME) + 4-5 fixture mancanti (destruct-reenter-
   PropSet, typed-coercion, clone, lazy-init-drop, gc-mid-arm) + predicato
   `is_gc_container` unico/esaustivo a due livelli.
3. **Misura peak (punto 4 WP-102, emendato)**: banda rumore full-peak
   della gamba PHPR (R≥5, ABAB, mediana+spread) PRIMA; poi A/B pin
   S-99↔S-100 a modo FISSATO off/off; bande UNILATERALI; spread ≥48 MiB ⇒
   bisect vietato.
4. **Census pila operandi** (push/pop per sito-opcode e primitiva) + leva-
   nulla di taratura del profilo; SOLO POI: H-C2 (banda [8,22], pavimento ½)
   e l'eventuale riscrittura slot-diretti (dump-diff prima del cronometro).
5. **Denti e igiene** (piccoli, bloccano la fiducia): A-HE-103-3 dente
   absent≡`=1` in sottoprocesso col dump-diff (sostituisce la metà
   tautologica); tripwire ON su corpo-zoo fuori-funnel (A-HE-103-1); gate
   fixture pinnato a 13 per NOME; expected-diff §3.13 path-normalizzato;
   gamba alloc a mem-census.
6. (timebox) fix §3.13 fedele (riga timbrata all'ACCODAMENTO come opline
   della lettura — Stogov A-ST-103-4; il fix CANCELLA la carve-out, gate
   fixture torna a diff zero) + H-D tavola completa se resta finestra.
   H-C1c resta GATED su fixture/giudici per specie (KS-KL-103-1).

**BACKLOG per NOME** (non slot di sessione): A-HO-103-2 (Generator:
birth-track prima), A-HE-103-2 (budget call-site `enabled()`), A-BA-103-4
/A-GR-103-3 dettagli census, 21,2% run_loop senza nome (dopo census pila),
A-KL-103-1 (staging), lezioni riscritte a segno ignoto (fatto in chiusura).

## Conflitti registrati

- predicato specie: Hoare (unico esaustivo, non-compilare su variante nuova)
  vs Stogov (due livelli Z_REFCOUNTED+COLLECTABLE) — COMPONIBILI, il team
  ricevitore li fonde (predicato unico che implementa i due livelli).
- pavimento H-C2: Bak (½ per «sovrastima 2×») vs Gregg (segno ignoto ⇒ il
  ½ non è derivabile) — atto salvo, motivazione riscritta (team misura).
- gating dell'apparato: Hoare severo (audit prima di tutto) vs Stogov
  (bastano le fixture) — composizione: audit+fixture entrambe in §2, prima
  di ogni ESTENSIONE del move, non della promozione fatta.
- verdetti S-101: Klabnik «regge» / Hejlsberg capitale sul dente /
  Pedersen riserva sul server — domini disgiunti, entrambe le capitali
  adottate senza contraddizione di merito.
