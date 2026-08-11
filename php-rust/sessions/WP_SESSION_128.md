# WP_SESSION_128 — rimisure a valle delle leve (full/media + compoff) + F2 caduta pulita

**In una frase**: rimisurato quanto il motore Rust si è avvicinato a PHP dopo le due
leve (WordPress intero ~1,77–1,80× nelle coppie pulite; primo `composer install`
misurato: ~1,88×); una terza leva tentata è più lenta e ritirata con prova al byte.

**SCOREBOARD** (pin **s127b ccb63dca INVARIATO**; micro = gate s127b, non rieseguite):
**arith 5,3 = · prop 5,6 = · calls 4,9 (*) = · str 4,2 = · arr 3,2 = · re 2,6 =**
((*) resta da osservare al prossimo gate) · **WP full = 1,758–1,909** (nuovo rif @ s127b;
coppie proprie ON 1,765–1,767; peak 2,299–2,374×, 1828–1894 MiB) · media 2,447–2,539 ·
**compoff 1,863–1,891 net (PRIMA cifra viva)** · **leve perf spedite: 0 — TENTATA 1
(L-OL1-F2, A/B eseguito, CADUTA)**; incidenti: 2 processo + 1 apparato. 2026-08-11.

## Esiti secchi
1·Pair109 ×4 gambe rc=0, parità per NOME esatta (solo wp_is_stream #2 ×4): **full
  1,758–1,909, media 2,447–2,539, NUOVO riferimento** (phpr ↓ su ogni gamba; bordo
  alto = denominatore, oracle spread 5,1%) → REPORT_GAP_128.
2·**compoff RIMISURATA E CHIUSA**: 1,863–1,891 net (raw 1,820–1,847), vendor_ok
  bilaterale, contesa ok (ictx/s), tarball ricongelato CON composer-x (1001 voci).
  Si posiziona accanto al WP full: I/O-densa (sys≈user), non object-dense.
3·**L-OL1 seg.2 istruttoria COMPLETA** (criterio f14a365 prima di tutto): census F1
  6/6 OK (Δins_alloc=5) · disasm (field_write 1297/92) · profilo · **sonde p2–p6**:
  overwrite in place +4 ⇒ costo FISSO per-statement (Vec chiavi di pop_field_keys).
4·**F2 «keys-scratch» NOMINATA → predizioni census 11/11 PRED-OK → smoke2 pulito
  D=−16,7 SEGNO OPPOSTO ⇒ CADUTA** (early-stop da criterio); revert 63f7688 provato
  AL BYTE (rebuild canonico = ccb63dca); server rilinkato per sbaglio e RIPRISTINATO
  dallo stash (bc95ba71). Meccanismo indiziato: bookkeeping scratch + seconda
  monomorfizzazione della filiera > ~2 mi_malloc/free small-path.
5·Incidenti: (proc.) mv/rm su file TRACKED ⇒ tree sporco, gate PRE morde — emenda:
  relitti fuori dal tracking (d8ea680); (app.) smoke1 con mediaanalysisd 202% visibile
  — contaminato conservato, rerun pulito. Morso nuovo: awk confronta hex tutto-cifre
  in NUMERICO (…e18 = scientifica) — forzato a stringa nel disasm.
## ⭐ Lezioni (max 3)
- ⭐⭐ Un census può confermare le predizioni alloc 11/11 e la leva cadere lo stesso:
  il conteggio allocazioni NON è un modello di tempo — il veto «alloc-removal senza
  modello del costo SOSTITUTIVO» esiste esattamente per questo e va onorato PRIMA.
- ⭐⭐ Gli output di run TRACKED nel repo sono mine vaganti: chi li muove/cancella
  sporca il tree e fa mordere i gate PRE a valle; la cura è toglierli dal tracking,
  mai allentare il gate.
- ⭐ Il check di quiescenza è un GATE, non una riga nello stesso comando del lancio.
