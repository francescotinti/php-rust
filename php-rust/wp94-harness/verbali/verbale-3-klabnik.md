# Verbale 3 — Klabnik (Concilio WP-94) — FORGE contro il gate cifre v3

Metodo: bite-test con controllo negativo. Ogni riga sotto è stata ESEGUITA
(giudice a HEAD `judge_sha=8f36d4dd967983c5`, head `e52a634a70fe`; selftest v3
**PASS in 13:30**; `--all` baseline **rc=1** per 8 verbali WP-94 untracked).
Doc forge in scratch fuori repo; nessun commit; albero porcelain
(solo `?? php-rust/wp94-harness/`).

## Forge

| # | Forge (eseguito) | Esito | Controllo negativo |
|---|---|---|---|
| **F-K1** | doc con `b_peak per worker = 6.910.767 B` (prefisso decimale della riga `rev=` WP-80) | **ADVISORY-PASS rc=64** — un'IDENTITÀ nominata legalizza una MISURA | `6.910.768 B` → FAIL «not in committed corpus» |
| F-K1b | `massa residua = 92.952.364 B` (prefisso del commit S-92.0) | **rc=64** (≈88,6 MiB, del tutto pubblicabile) | idem sopra |
| **F-K2** | `picco per il worker W16-20.999.999 B` | **rc=64** — lo stripper di identificatori MANGIA `W16-20` e il resto `.999.999` è invisibile al tokenizer (lookbehind `.`) | `20.999.999 B` nudo → FAIL |
| F-K2b | `b_peak-20.999.999 B per worker` (forma etichetta-trattino, la più pubblicabile) | **rc=64** | idem |
| **F-K3** | `19.922.944 B = 19,00 MiB [derivata: prov 24313856@…m88.slope.w16.r4.a1.memcensus:40 − 4390912@…:47]` — stessa chiave `reserved=`, **righe/istanti diversi** | **rc=64** + «A-SK60 provenance-verified» | senza tag → FAIL 2 token |
| **F-K4** | `bash -c "$(cat patched.sh)" gate-measure-cifre.sh --all` da `wp81-harness` (giudice patchato: dente perimetro disarmato) | **rc=0** + `PASS gate-measure-cifre --all … judge_sha=8f36d4dd967983c5` | STESSO testo come file `zzforge-k5.sh` → `REFUSE … (A-SK-78)` |
| **F-K5** | `.MD` maiuscolo con cifra fabbricata dentro `php-rust/wp94-harness/` + `--all` | **evaso**: nessuna riga FAIL lo NOMINA (8 FAIL prima, 8 dopo) | il gemello `.md` è notoriamente morso (T16) |
| F-K6 | advisory su `ROOT/COVERAGE.md` e `ROOT/sessions/WP_SESSION_75.md` (fuori dal complemento `^php-rust/.*\.md$`) | FAIL **11** e **4** cifre non giudicate, invisibili a `--all` | i gemelli `php-rust/COVERAGE.md` hanno riga manifest |

## Misure a corredo (replica esatta del corpus: cardinalità riprodotta 24.329/24.329)

- Corpus: 916 sorgenti, **cardinalità 24.329 = budget 24.329** (headroom **0**).
- Pool operandi A-SK-81: 616.778 occorrenze etichettate; **3.113.808** coppie
  stessa-chiave/stesso-file → **418.116** differenze distinte, **163.175** nella
  finestra 1–99 MB; chiusura **50/100** dei target MiB-tondi (k·1.048.576,
  k=1..100) e 5/100 dei 1M-tondi. L'etichetta vincola la GRANDEZZA, mai l'ISTANTE.
- Identità: 1.447 antenati, **54** con prefisso decimale ≥7 cifre (33 a 8);
  le 2 righe `rev=` in vigore legalizzano 3 token, spendibili come byte.
- Perimetro: 238 `.md` sotto `php-rust/` (238 righe manifest) contro **22 `.md`
  committati FUORI** (README/COVERAGE/TODO **di root — quelli che GitHub
  pubblica**, `sessions/WP_SESSION_75.md`, `diary/*`), **894** occorrenze di
  token ≥3 cifre mai giudicate.

## VERDETTO

Il v3 **non regge**: quattro rifiutazioni capitali. A-SK-78 firma un OMONIMO A
UN PATH, non il testo che gira (`$0` è scelto dal chiamante); A-SK-76 rifiuta la
colla a destra e la lascia intatta a sinistra; A-SK-81 confonde «stessa chiave»
con «stesso istante»; A-SK-80 chiama «complemento» un sottoalbero. In più T16 è
**vacuo oggi**: asserisce solo `rc==1` da `--all`, e la baseline è già 1 ogni
volta che esiste un `.md` untracked (cioè in ogni sessione di concilio).

## Emendamenti

- **A-SK-82** (tether reale): ancorare a `${BASH_SOURCE[0]}` e REFUSE se vuoto o
  ≠ `$0` — verificato: `bash -c`/`bash -s` danno `BASH_SOURCE` VUOTA, un file
  vero la valorizza. Dente: F-K4 come tooth permanente T23.
- **A-SK-83** (colla a sinistra): un run ≥3 cifre ADIACENTE a un identificatore
  (`[-_.]`) è RIFIUTATO, mai cancellato — simmetrico ad A-SK-76 (T24).
- **A-SK-84** (istante): operandi prov dalla STESSA RIGA, o da righe con
  marcatore d'istante uguale dichiarato; la chiave sola è insufficiente (T25).
- **A-SK-85** (identità ≠ misura): token `rev=` citabile solo senza
  raggruppamento italiano e mai adiacente a un'unità (B/MiB) (T26).
- **A-SK-86** (perimetro vero): ogni `.md` del REPO, regex **case-insensitive**;
  le 22 esclusioni fuori `php-rust/` per NOME nel manifest (T27 = F-K5).
- **A-SK-87** (denti per NOME): ogni dente su `--all` asserisce la RIGA FAIL che
  NOMINA il forge, previa misura della baseline; T16 riscritto.

**Refutazioni capitali: SÌ** — KS-SK-94-1 (tether/F-K4), KS-SK-94-2
(stripper/F-K2), KS-SK-94-3 (istante/F-K3), KS-SK-94-4 (perimetro/F-K5+F-K6);
KS-SK-94-5 = vacuità di T16 e identità-come-misura (F-K1).
