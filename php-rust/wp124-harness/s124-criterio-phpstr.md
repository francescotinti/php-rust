# s124-criterio-phpstr.md — PhpStr single-alloc: CRITERIO (scritto e committato PRIMA della patch)

## Criterio (≤10 righe, REGOLE §3)
1. LEVA: `ZStr = Rc<PhpStr{Cell,Vec}>` (2 malloc/stringa) → blocco UNICO `{rc,hash,len,cap}+bytes` (1 malloc, header 32 B invariato), refcount custom; funnel zstr.rs:54; concat2 e concat_n_join costruiscono DIRETTO nel blocco (niente Vec transiente).
2. SEGNO atteso sul BERSAGLIO **str**: D_med > 0 (B più veloce); attesa lorda −2 GA-alloc/iter × ~14 ns (prezzo L-RE1) ≈ +28 ns/iter.
3. ADMISSION PRIMA del tempo: census fuso stesso head (ricetta classifica-v2), predizioni: str −2 · re −3 · arr ~−2 alloc/iter; predizione mancata in grado ⇒ STOP e ridiagnosi, niente A/B.
4. A/B: A = pin 885d2c64 (stash s120-re1), B = patch con ricetta A′; file SCALATI wp123 + timer ucpu µs (unico perimetro in cui le bande v2 valgono — az. rev. #4); ordine ALTERNATO per coppia, R=5, EXTRA_MAX=3, floor med3 per binario, parità output su tutte le 6 categorie.
5. SOGLIE (SOLO metro v2): promo str = +max(4,00; SL 2,89; 2×spread_A; 4×quanto) · refut = −max(SL 2,89; 2×spread_A; 4×quanto); N dal TSV del sorgente.
6. GIUDICE: `s124-giudice-v3.sh` = v2 con **BSTOR e ZAVORRA RITIRATI** (az. rev. #3): guardie non-bersaglio arith/prop/calls a SOLO-regressione con soglia −max(SL; 2×spread_A; 4×quanto); rc 0/1/2/3 su file.
7. SECONDARI re/arr: direzione attesa + nella STESSA misura (A/B proprio, 6 categorie): magnitudine dichiarabile per ciascuna dal proprio D_med; nessuna ripartizione tra categorie.
8. GATE promozione (rc=0): batteria `cargo test --release` (rc dal comando) · corpus 1415×2 per NOME · fixture bilaterali · ricetta ORM/http-kernel (php-types toccato) · pin nuovo SOLO via scripts/pin-phpr.sh; scoreboard vs oracle con micro ORIGINALI richiede rimisura dedicata (arr in testa — az. rev. #4).
9. RISCHI SEMANTICI (non temporali, istruttoria s123): RcEqIdent chiavi ⇒ PartialEq manuale ptr-fast-path; hash cached ⇒ Cell nel header via &self; !Clone strutturale ⇒ il tipo custom non espone Clone dell'header. Arbitri: batteria + corpus per NOME (i lookup di chiave vivono lì).
10. ESITI: rc0 = promozione ai gate · rc1 = refutazione (candidato a reperto) · rc2 = non distinguibile · rc3 = misura invalida; qualunque esito si scrive nel verdetto .out con numeri.

## Modello del COSTO SOSTITUTIVO (obbligo istruttoria: prezzato PRIMA del tempo)
| Cosa sparisce | Cosa lo sostituisce | Prezzo modello |
|---|---|---|
| 2° malloc+free (RcBox+buffer → blocco) | — | −~14 ns/alloc rimossa (L-RE1), solo path slice-fed/diretti |
| 2 load dipendenti (Rc→PhpStr→buf) | 1 load (offset fisso header+32) | ≥0 guadagno; header condivide la cache line coi primi byte |
| Rc inc/dec (strong) | `Cell<usize>` inc/dec + branch a zero | Δ≈0 (stesse istruzioni; niente campo weak da mantenere) |
| RcEqIdent (ptr‖byte, specializzazione libstd) | PartialEq manuale ptr-fast-path‖byte | Δ≈0 SE scritto; se omesso regredisce arr (guardia/bersagli la vedono) |
| `Rc::get_mut` (`.=` WP-55) | check rc==1 + realloc blocco intero (header+bytes), crescita ammortizzata ×2 | Δ≈0 sul path append (+32 B copiati per regrow, ammortizzato); canale O(n²) RESTA morto |
| Vec-fed `new(vec)` (buffer CEDUTO, zero copy) | memcpy(len) nel blocco | **COSTO NUOVO O(len)** sui path Vec-fed residui (es. file_get_contents); conteggio alloc invariato (2→2). Le micro sono slice-fed/dirette; il full WP (p.4 NEXT) resta l'arbitro. Le 4 cadute alloc-removal avevano ESATTAMENTE costi sostitutivi O(n) non prezzati: questo è dichiarato ORA. |

Perimetro patch: zstr.rs riscritto + 7 siti Rc-API (run.rs:432 get_mut/clone · ops.rs:631 ptr_eq · zval.rs:375 test · memcensus.rs:357/376 as_ptr · vm/mod.rs:1260/1843 as_ptr) + ~14 `Rc<PhpStr>` testuali → `ZStr`; il compilatore è la rete per i siti UFCS residui (`Rc::clone` ecc.).
