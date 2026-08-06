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
- Annotazioni `scn!(Sito: DropS/C = n)` SUL SENTIERO ESEGUITO nei punti
  dove la vita di uno Zval finisce dentro l'arm. Enumerazione STATICA
  (da arbitrare col dinamico, lezione S-102 «lo statico stavolta ha
  retto»):
  | sito | drop attesi/iter | che cosa muore |
  |---|---|---|
  | Pop ×2 | 2 | il valore scartato (650) |
  | PropGet ×2 | 2 | l'handle ricevitore mosso, a fine arm |
  | PropSet ×1 | 2 | l'handle ricevitore + il vecchio valore (post gc_note) |
  | BinaryAdd ×1 | 1 | rhs poppato consumato (fast Long) |
  | BinaryDst ×1 | 2 | rhs+lhs poppati consumati |
  | CmpJmpSC ×1 | 1 | il const materializzato `cv` (1186) |
  | IncDecSlot ×1 | 1 | il gemello non pushato di old/new (992) |
  | **TOTALE statico** | **11** | — la stima «~11» esce ESATTA dalla conta |
- ATTESA PRE-REGISTRATA da confermare dal dinamico: 11 drop/iter di cui
  **DropC = 2-3** (handle ricevitore ×2, old di PropSet se container) e
  il resto DropS. Linearità 300:1 obbligatoria (prop_small).
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
