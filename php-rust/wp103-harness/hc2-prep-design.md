# H-C2 — design dei PREFISSI vincolati (S-103 punto 3, PRIMA di ogni leva)

Sequenza VINCOLATA (KS-BA-104-2/3, KS-MA-104-1, KS-HE-104-1):
leva-nulla → drop-census → `hc2-criterio.out` → A/B da sola → gate pieno.
Questo file è il DESIGN dei primi tre passi; i numeri arrivano solo
dall'esecuzione (niente attese in ns: KS-GR-103-1).

## 1. Leva-nulla (banda-LAYOUT, A-BA-103-4 → KS-BA-104-2)

Perché: ABAB è cieco al code-layout — una leva micro sotto la banda di
layout misurerebbe il linker, non la leva. UNA build, due letture
(composizione Bak/Gregg): taratura layout E calibro del profilo.

- Perturbazione SEMANTICAMENTE NULLA: funzione `#[inline(never)]` morta
  ma non eliminabile (chiamata dietro un check env che a runtime è
  sempre falso, es. `PHPR_NULL_LEVER=1` mai settato nel giudice),
  inserita PRIMA di `run_loop` nell'unità — sposta il layout del testo
  caldo senza toccare un solo sentiero eseguito.
- Misura: giudice prop (R=5, mediana, netto pavimenti) pin ↔ build
  leva-nulla, stessa sera, ABAB. |Δ| = **banda-layout**; qualunque A/B
  micro successivo è giudicabile SOLO se |Δ_leva| > banda-layout.
- Dump-diff bimodale pin↔leva-nulla DEVE essere zero (semantica nulla
  provata, non presunta).

## 2. Drop-census (contare gli ~11, KS-BA-104-3 ≡ RC-LE-104-1)

Canale mai contato = fuori dai criteri. Estensione MINIMA di
`stackcensus` (stesso pattern S-102, feature `zval-census`):

- `Prim::DropS = 5` («drop scalare») e `Prim::DropC = 6` («drop
  container») — la SPECIE è il canale della leva (fast-out scalare):
  contare le due classi separate dà direttamente il numeratore
  indirizzabile. Classificazione col predicato ESISTENTE
  `is_gc_container` (mai un secondo predicato: KS-MA-104-1).
- Annotazioni `dcn!(Sito: &v)` SUL SENTIERO ESEGUITO nei punti dove la
  vita di uno Zval finisce dentro l'arm. Enumerazione STATICA — v2,
  RAFFINATA in S-103 leggendo i corpi PRIMA del dinamico (la v1 da
  memoria contava 11 e mancava gli OVERWRITE: anche un bersaglio
  sovrascritto droppa il vecchio valore):
  | sito | drop attesi/iter | che cosa muore |
  |---|---|---|
  | Pop ×2 | 2 S | il valore scartato (scope end) |
  | IncDecSlot ×1 (fast) | 1 S | il vecchio Long dello slot, sovrascritto |
  | BinaryAdd ×1 (fast) | 2 S | rhs consumato + il vecchio top sovrascritto |
  | BinaryDst ×1 | 3 S | lhs+rhs consumati + il dst sovrascritto |
  | CmpJmpSC ×1 | 2 S | il const materializzato `cv` + il Bool `res` |
  | PropGet ×2 | 2 C | l'handle ricevitore mosso, a fine arm |
  | PropSet ×1 | 1 S + 1 C | il vecchio valore (Long nel giudice) + l'handle |
  | **TOTALE statico v2** | **14 = 11 S + 3 C** | la stima «~11» combacia con i SOLI scalari |
- ATTESA PRE-REGISTRATA (v2, da confermare dal dinamico): **DropS=11 e
  DropC=3 per iter** — il canale della leva fast-out è esattamente il
  DropS. Linearità 300:1 obbligatoria (prop_small). Se il dinamico
  smentisce, fa fede il dinamico e la tavola si emenda CON NOME.
- Il dump appende `drop_s=`/`drop_c=` alle righe stackcensus (formato
  census-only, nessuna build di parità la vede).

## 3. `hc2-criterio.out` (dopo il census, PRIMA della leva)

- Banda [8,22] ns/iter del Concilio WP-103 RIPESATA sul CONTATO
  (drop_s effettivi × costo/drop da Δ_A/B della leva-nulla? NO — il
  costo/drop fa fede SOLO dall'A/B della leva stessa ÷ drop contati).
- Pavimento ½ prudenziale; pin `size_of::<Zval>()` nel criterio
  (KS-HE-104-1: se la leva cambia la taglia, il criterio è VOID).
- Fast-out SOLO via `is_gc_container` (A-HO-104-5 ≡ A-MA-104-4): un
  fast-out che salti drop/gc_note su un container = reject.
- Δ si divide per SITO×PRIMITIVA (KS-BA-104-1), mai ÷11 aggregato.
