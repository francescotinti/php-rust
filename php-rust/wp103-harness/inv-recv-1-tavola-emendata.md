# INV-RECV-1 — TAVOLA EMENDATA (S-103 punto 2, Concilio WP-104)

**Mandato**: RC-HO-104 (l'audit S-102 prova l'invariante SOLO sotto base=2)
+ RC-MA-104 (le fixture 17-18 a distanza ≥2 dalla soglia non arbitrano il
−1) — ordine §2 di `wp104-harness/verbali/SYNTHESIS.md`. Questo file
EMENDA `wp102-harness/inv-recv-1-audit.md` senza riscriverlo (la storia
resta): l'esito lì dichiarato si legge d'ora in poi con le restrizioni e
le righe qui sotto.

## Esito RISTRETTO (sostituisce il §Esito dell'audit S-102)

**L'audit statico S-102 prova l'invariante per ricevitori SLOT-HELD
(base=2: created + slot)**. La contabilità «mid-arm ≥ soglia+1 in entrambi
gli schemi» usava base=2 in ogni riga: per base=1 (ricevitore temporaneo,
es. `(new C)->x`) il conteggio mid-arm post-move scende a **2** ed entra
nella zona degli osservatori `==2` (OBS-4, OBS-6) e dell'aritmetica esatta
del collector (OBS-8) — casi NON coperti dall'audit. Quella zona è ora
arbitrata DINAMICAMENTE (righe sotto), non staticamente.

## Marcatori STABILI degli osservatori (d'ora in poi si citano così)

I numeri di riga dei sorgenti churnano: gli osservatori si nominano
`OBS-1..OBS-12` con la mappa qui sotto (riferita all'audit S-102, che
resta la descrizione integrale). I commenti-marcatore in codice ai siti
entrano col primo commit runtime di S-103 (assert Ref, A-ST-104-4).

| marcatore | sito (audit S-102) | soglia | perimetro |
|---|---|---|---|
| OBS-1 | oop.rs `last_user_ref` (~1084) | `== 2 + extra` | classe A |
| OBS-2 | oop.rs release closure (~1091/1100) | `== 1` | classe A |
| OBS-3 | oop.rs cella Ref (~1096) | `== 1` | FUORI (osserva la cella) |
| OBS-4 | mod.rs buffer collectable-ADESSO (~4126) | `== 2` | classe A — **zona base=1** |
| OBS-5 | mod.rs releasable in sweep (~4160) | `− extra == 1` | classe A |
| OBS-6 | mod.rs `exclusive` cascade (~4458) | `== 2 && …` | classe A — **zona base=1** |
| OBS-7 | mod.rs gc_cascade destructed (~4554) | `== 1` | classe A |
| OBS-8 | mod.rs collector external-holder (~4914-4918) | `− 2 > in_edges` | classe A — **aritmetica esatta** |
| OBS-9 | mod.rs dtor-dead (~5252) | `== 1` | classe A |
| OBS-10 | run.rs cella static (~742) | `== 1` | FUORI (cella static) |
| OBS-11 | mod.rs typed_refs (~14155-14157) | `> 0` / `> 1` | classe A (soglie `>`) |
| OBS-12 | mod.rs gc_note descend (~3946/3974/3989) | `== 1` | FUORI (osserva old) |

## Righe NUOVE (arbitrato dinamico, S-103)

| riga | contabilità mid-arm (pre → post) | osservatori toccati | arbitro | verdetto |
|---|---|---|---|---|
| **base=1** (ricevitore temporaneo `(new C)->x`) | 3 → **2** (created + handle mosso) | OBS-4/OBS-6 (`==2` toccato ESATTO), OBS-8 (post: 3−2=1 col self-cycle) | **fixture 19b** (`recv-fixtures/19b-base1-ricevitore-temporaneo.php`): collector MID-ARM sul temp in self-cycle — se l'handle mosso non conta da holder esterno, il temp è raccolto in volo (dtor anticipato) | **PASS** byte-id oracle ×2 modi sul pin d0b01362 (primo colpo; correzione cosmetica echo registrata) |
| **soglia esatta su slot-held** (slot morto DENTRO il braccio) | 4 → **3** al pop; poi lo slot muore mid-arm ⇒ 3 → **2** | OBS-8 (post: 3−2=1 > in_edges=1 FALSO se l'handle non conta), OBS-4 in coda | **fixture 19a** (`recv-fixtures/19a-soglia-esatta-slot-held.php`): `__get` azzera lo slot poi collector — il −1 del MOVE è arbitrato a distanza ZERO dalla soglia | **PASS** byte-id oracle ×2 modi sul pin d0b01362 (primo colpo; correzione cosmetica echo registrata) |

Gate: `s103-recv-fixtures.sh` (pinnato ai 2 NOMI o VOID), out in
`recv-fixtures/out/`. Run S-103: `recv_fixtures_fail=0`.

## Che cosa QUESTO emendamento sblocca e che cosa NO

- SBLOCCA (insieme al resto del pacchetto §2): le ESTENSIONI
  MOVE/H-C1c restano gated dai loro KS (KS-MA-103-2/3, KS-ST-103-2) ma
  non più dal buco base=1/soglia-esatta.
- NON tocca: le promozioni H-C1a/b (mai bloccate — RC-MA-104: nessun
  osservatore mid-arm nel giudice; corpus/fixture coprono l'uso).
- Resta dovuto nel pacchetto: **assert Ref in `is_gc_container`**
  (A-ST-104-4, col primo commit runtime) + marcatori OBS in codice.
