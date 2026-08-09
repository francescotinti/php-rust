# WP_SESSION_119 — C-lite ESEGUITA (tabella 6×4) · treno-2 V3-V5 PROMOSSO (pin s119) · server s119 gradato · coppia WP full 1,716/1,837

**In una frase**: abbiamo contato, per la prima volta su ENTRAMBI i motori, quante
operazioni di ciclo-di-vita fa ogni categoria (il residuo delle proprietà non è
memoria ma copie), promosso tre ritocchi che evitano una copia del ricevitore, e
il carico WordPress completo scende sotto 1,9× in entrambi i modi.

**SCOREBOARD** (pin s119 **350582e5** @ 22e0cda = A′+L-A+H-P1+V3-V5; micro R=5;
frecce vs s118): **arith 5,4 ↓ (5,5) · prop 5,5 = · calls 5,0 ↑ (4,8) · str 5,6
↑ (5,3) · arr 3,7 ↓ (3,8) · re 3,3 =** (calls/str: A/B stessa-sera dice guardie
TENGONO — calls D_med −0,50 = quanto layout, str D_med +5,00; le frecce dei
rapporti includono il rumore oracle) · WP sul pin s119+server s119: **full ON
1,716 / OFF 1,837 · media 2,636/2,519 · peak ON 1933,6 MiB** (serie A′ N=1,
direzione-solo; oracle mosso ~7% tra le gambe). **Leve perf spedite: 1
(treno-2 = V3+V4+V5)**. 2026-08-09 · Fable 5 · 8bcd7d4→cd2c4a3.

## Esiti secchi
1·**C-lite eseguita** (criterio PRE 00a4824, 4 deroghe NOMINATE; verdetto
s119-clite-verdetto.out): tabella 6×4 R=2 deterministica; int-pure ZERO alloc su
entrambi i motori ⇒ il residuo prop è CICLO-DI-VITA (5 cloni Zval/iter, 1 Rc, vs
0 rc-op Zend); delta alloc: **re +12/iter · str +3 · arr +2** = candidati per NOME.
2·**Incidente deroga-4 STOP, curato col meccanismo firmato**: qualunque edit a
php-types (anche cfg-gated: split-derive 7d104a91, one-piece c38037d6) cambia il
binario (span→svh→nomi simbolo); sorgenti RIPRISTINATI ai byte del pin
(15dfb6b3 riprodotto), strumentazione spostata in `census-clite.patch`.
3·**treno-2 PROMOSSO** (criterio PRE fd2e8d3): guardia-Ref sul ricevitore
POSSEDUTO in PropIncDec/PropIsset/MethodCall; admission 6/6+6/6 dump; guardie
A/B 6/6 TENGONO; held-out **N=3** 3/3 con **poly −0,13 s** (direzione attesa,
banda N3 0,04); §6 pieno via **resume dichiarato** (interruzione a build in
corso, nessun passo saltato): batteria 1742/0/2 inventario IDENTICO, corpus
canonico rc=0, fixture 6/6 → pin s119. 4·Server **b7bd6744 pin s119 GRADATO** ×2
modi (option 413+restapi 3508 per NOME, 0/0) · coppia WP rc=0 ×2 (== wp_is_stream).

## ⭐ Lezioni (max 3)
- ⭐⭐ **Nessun edit a php-types è gratis**: span→svh→simboli — la strumentazione
  del pin vive in PATCH di harness, mai nei sorgenti committati.
- ⭐⭐ **Lo smoke post-build può mentire**: prop −1,67/−2,67 a R=2 svanito a R=5
  (+0,00) — primo-giro; lo smoke di guardia registra, la banda giudica.
- ⭐ **Una cerimonia interrotta si riprende dal passo dichiarato**, mai da capo e
  mai saltando: il resume ha rifatto build→batteria→pin con gli stessi gate.
