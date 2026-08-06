# Concilio WP-105 — INDICE (su S-103 e programma S-104)

Formato indice (decisione utente 2026-08-06): i testi INTEGRALI vivono
SOLO in `verbali/`; qui link + ricevute ≤80 parole. Sintesi:
[`verbali/SYNTHESIS.md`](verbali/SYNTHESIS.md) — **VINCOLANTE per S-104**.
Esito in sessione: 9/9 CON EMENDAMENTI, nessun MI OPPONGO; 1 refutazione
capitale CONVERGENTE (Hoare∧Bak: predicato fast-out) + 7 maggiori; regola
di lettura A/B R=7 PRE-REGISTRATA in sintesi col run ancora in volo.

## Fase 1 — sedie

| # | Sedia | Verbale | Ricevuta |
|---|---|---|---|
| 1 | Hoare | [verbale-1-hoare.md](verbali/verbale-1-hoare.md) | CON EMENDAMENTI — A-HO-105-1 predicato trivial-drop distinto; -2 assert issato; -3 doc; -4 braccio rosso 19a/b — KS-HO-105-1 fast-out su !is_gc_container=reject; -2 leva chiusa finché -1 non consumato — CAPITALE: criterio conflava non-container con drop-banale (Str/Resource/Generator) |
| 2 | Matsakis | [verbale-2-matsakis.md](verbali/verbale-2-matsakis.md) | CON EMENDAMENTI — A-MA-105-1 mutation-check 19a/b (rosso archiviato); -2 fixture 19c hook; -3 ledger fine-vita; -4 dente panic fast-out — KS-MA-105-1 sabotaggio non morde⇒RC-MA-104 si riapre; -2 ledger diverge⇒attribuzione VOID — capitali: no; «arbitro mai visto fallire non arbitra» |
| 3 | Klabnik | [verbale-3-klabnik.md](verbali/verbale-3-klabnik.md) | metodo solido ma parity-null solo DICHIARATO — R1 oracle non pinnato; R2 mode-probe falso-OK; R3 «fa fede HEAD» non è pin; R4 baseline micro su binario ≠ pin chiusura; R5 cross-mode stantio — A-KL-105-1..5, KS-KL-105-1/2 |
| 4 | Hejlsberg | [verbale-4-hejlsberg.md](verbali/verbale-4-hejlsberg.md) | approvata con riserve — ==1 conflaziona tre cause (per-corpo); =0 senza controllo positivo (recidiva); zoo incompleto; size==16 non pinna align; banda-layout N=1 accidentale — A-HE-105-1..4, KS-HE-105-1..3 |
| 5 | Bak | [verbale-5-bak.md](verbali/verbale-5-bak.md) | prefissi solidi; leva ADDITIVA ma CAPITALE: is_gc_container come predicato fast-out = leak di ogni stringa poppata; banda-layout campione R=1; [8,22] mai misurata al numeratore (serve disasm) — A-BA-105-1..4, 3 KS |
| 6 | Pedersen | [verbale-6-pedersen.md](verbali/verbale-6-pedersen.md) | 31aa7c2e resta MINIMO ma perimetro RISTRETTO (cbE mai cross-worker/a freddo; confine=solo body); symlink a catalogo; stash meccanico nel launcher — KS-PE-105-1 zero cifre senza PIENO; il PIENO sul pin post-leva |
| 7 | Leijen | [verbale-7-leijen.md](verbali/verbale-7-leijen.md) | approvata con riserve — 32,0 B solo soffitto-alloc (free ASSUNTO, serve free-hist); Rc<RefCell<Zval>>=40B ⇒ ret_cell escluso per layout; R=7: spread monotono in R ⇒ sign test co-primario — A-LE-105-1..5, 3 KS |
| 8 | Stogov | [verbale-8-stogov.md](verbali/verbale-8-stogov.md) | REFUTO: §3.12 ha TRE regimi (strict e .= CONSERVANO — censimento 4/4 monoregime); §3.13 marca perde l'UNITÀ (include/eval divergono, PROVATO su HEAD); generator fedele = get_gc COMPLETO; A-ST-104-4 sciolta — 5 emendamenti, 3 KS |
| 9 | Gregg (mandato inverso) | [verbale-9-gregg.md](verbali/verbale-9-gregg.md) | onesta ma soglia anomalia RAGGIUNTA (2 sessioni ferme); banda tra-sere: 2 punti stesso giorno = 1; N auto-emesso dal giudice; R=7 ultimo tentativo poi per-fase — KS-GR-105-1: senza A/B H-C2 in S-104 ⇒ contatore 3, riallocazione |

## Fase 2 — team

| Team | Nota | Ricevuta |
|---|---|---|
| leva (Hoare+Bak) | [team-leva.md](verbali/team-leva.md) | convergenza capitale identica ⇒ `is_trivial_drop` (match esaustivo) + dispose unico + dente cross-check + fixture stringhe-in-Pop; nessun conflitto; sequenza: emenda-criterio atto zero, disasm 30′, leva con gate pieno; banda-layout N≥3 solo se Δ marginale |
| misura (Leijen+Gregg) | [team-misura.md](verbali/team-misura.md) | regola lettura R=7 composta e PRE-REGISTRATA (tetto R-coerente; \|Δ\|+sign test co-primari; VOID⇒per-fase DOPO H-C2, mai terzo rerun); H-D: free-hist+attese byte-per-tipo prima del SiteTag; banda ≥3 punti ≥2 giorni; KS-GR-105-1 testuale nell'ordine |
| fedeltà (Stogov+Pedersen) | [team-fedelta.md](verbali/team-fedelta.md) | catalogo SUBITO (tre regimi §3.12, unit §3.13+probe, symlink); launcher A-PE-105-1/3/4 al grading del pin post-leva; fix generator/§3.12 solo se punto-fedeltà scelto, coi KS; nessun conflitto |
| gate (Klabnik+Matsakis+Hejlsberg) | [team-gate.md](verbali/team-gate.md) | pacchetto DENTI-105 «nessun arbitro senza rosso» (mutation-check + ogni braccio provato + zoo pinnato all'emettitore); regola PIN-105 (bilaterale, sequenza atomica, stash contestuale); parity-null funzionale E strumentale; distinzione blocca/backlog |
