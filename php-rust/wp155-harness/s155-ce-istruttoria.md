# S-155 istruttoria ce-count — «k=1, non 0» SPIEGATO AL SORGENTE (clausola criterio p.3: k>0 ⇒ sorgente)

## Esito in una riga
Il residuo 1 alloc×32 B/chiamata NON è del lookup CE1: è l'**args-Vec di
`pop_keys`** attribuito al nome dal census (lo scope `s149_name_scope` apre
PRIMA di `pop_keys` — `vm/run.rs:3655-3658`). **Il hit-path CE1 è a 0 alloc
CONFERMATO**: il claim della leva REGGE; era l'ATTESA k=0 a essere mal posta
(ignorava il termine di plumbing nell'attribuzione per-nome).

## Contabilità (tutta INTERA ESATTA)
- Driver ce-count (2 chiamate/iter, autoload=false, 2 argomenti):
  pre-CE1 k=2 = to_vec(1) + args-Vec(1) [la LcKey nel ramo false ESISTEVA già
  pre-CE1: diff cea0e8f] · post-CE1 k=1 = args-Vec(1); b=32 B = Vec di 2
  Zval×16 B. Δ misurato −1 == to_vec rimosso (edit a).
- **Controllo empirico** (`fe-count.php`, function_exists('strlen'): corpo
  0-alloc al sorgente — is_name_callable = find_fn_ci/LcKey/tabelle):
  k=1 INTERO ESATTO (Δn=200000/ΔN=200000), **b=16,0 B** = Vec di 1 Zval —
  l'attribuzione args-Vec è confermata su un builtin INDIPENDENTE.
  Raw agli atti: sonda-out/fe-{100000,300000}.raw.
- Il canale coincide con H-D (S-103: 1 alloc×32,0 B/chiamata hostcall,
  indiziato args-Vec S-104): stessa cifra, ora col sito NOMINATO.

## Correzioni alle attese di NEXT (da recepire in rotazione)
1. «ce-count k atteso 0» → il k corretto era **1** (plumbing attribuito).
2. «testa class_exists 9,74M → ~0» → attesa post-CE1 ≈ **chiamate×1**
   (args-Vec resta attribuita al nome).
3. «≈4,87M chiamate» (da k=2) → le chiamate ORM dipendono dal MIX di forme:
   1-arg autoload=true ⇒ k_pre=3 (args-Vec+to_vec+lowercase) · 2-arg
   autoload=false ⇒ k_pre=2 ⇒ **chiamate ∈ [3,25M; 4,87M]**; si risolve solo
   col census ORM al probe s155.

## OLTRE-attesa ORM (Δ +0,41/+0,50 vs attesa 0,03–0,07) — meccanismo candidato NOMINATO
Attesa micro-fondata: 3,25–4,87M chiamate × 15–22 ns ≈ **0,05–0,11 s** ≪ Δ.
Ma l'edit (b) di CE1 alleggerisce `resolve_class_autoload`, che è il funnel
di **11 siti** (host.rs:3132/7331/7656 · mod.rs:8494/8748/8809/9360/9460/
11953 · oop.rs:977: instanceof-stringa, new $class, catch-type, reflection,
interface_exists, …): ogni risoluzione classe-per-nome dell'ORM risparmiava
lowercase-alloc + byte-loop anche FUORI da class_exists. Direzione+meccanismo
firmati (CE1 unica leva s153→s154); **magnitudine NON ripartita** senza un
census ORM nuovo (REGOLE §4) — la gamba census col probe s155 è la sonda
naturale (apertura per NOME, già in coda).
