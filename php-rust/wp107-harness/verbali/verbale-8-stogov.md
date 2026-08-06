# Verbale sedia 8 — STOGOV (Zend engine / opcache, semantica PHP 8.5) — Concilio WP-107 su S-105

Fase 1 indipendente (rilancio post-limite API; verbali altrui NON letti).

**VERDETTO: CON EMENDAMENTI.** Nessuna refutazione capitale: la leva H-D
forma 2 è sana e coerente con Zend. Ma una narrativa del report è falsa al
dato, e la cura §3.15 va emendata prima di implementarla.

## (a) Forma 2 vs modello Zend — R-ST-107-1, R-ST-107-2, A-ST-107-1

**R-ST-107-1 — «converge al modello Zend» è vero A METÀ.** In Zend
INIT_FCALL alloca il call frame del callee PRIMA della valutazione degli
argomenti; ogni SEND_VAL/SEND_VAR scrive UNA volta direttamente in
ZEND_CALL_ARG; RECV fa solo default/verify. La forma 2 (run.rs:2594-2610)
elimina il contenitore ma CONSERVA il doppio transito push→pop sulla pila
del chiamante: 2 move/arg contro 1 write/arg di Zend. È un'approssimazione
con debito NOMINATO — e il debito residuo è esattamente la valuta di S-104
(volume di lavoro per op). **A-ST-107-1**: ritirare il doppio transito
richiede il frame-in-costruzione alla Zend (catena EX(call), cleanup del
frame parziale su eccezione a metà SEND, rientranza per chiamate annidate
negli argomenti): leva futura legittima, MAI «rifinitura» — protocollo
pieno + fx21.

**R-ST-107-2 — ordine di decay**: Zend dereferenzia alla SEND
(sinistra→destra); la forma 2 decade al pop (destra→sinistra). Equivalente
SOLO se decay_arg non esegue mai codice utente — oggi vero per costruzione
(clone Zval = bump Rc, niente __clone/__destruct). Concordo con
A-MA-107-1: il dente deve fissare QUESTO standard («mai user code nel
decay»), non genericamente «pure-read». **KS-ST-107-2**: run.rs:2606-2609
e il braccio fast di bind_params sono DUE copie della stessa semantica —
ogni futura modifica a decay/bind le tocca ENTRAMBE, o il pin fx21
PRE≡POST è VOID.

## (b) §3.15 — R-ST-107-3, A-ST-107-2

**R-ST-107-3 — indiziato CONFERMATO al codice, cura Zend-ESATTA
verificata.** Il push-side è compile/expr.rs:1616
(`by_ref.get(i)…unwrap_or(false)`): oltre vslot risponde false → by-value.
Il binder (calls.rs, ramo `variadic_by_ref`) appende senza decay: corretto.
L'oracle 8.5.7 (zend_compile.h:1143, `zend_check_arg_send_type`) clampa
`arg_num = num_args` sotto ZEND_ACC_VARIADIC ⇒ «posizioni ≥ vslot usano il
flag di vslot» è ESATTAMENTE il modello Zend (SEND_PREFER_REF è solo
internals, nessun analogo userland serve). **A-ST-107-2 — emendamenti**:
(i) il clamp copre TUTTO il ramo by-ref di push_call_args: letterale a
posizione ≥ vslot ⇒ Error runtime «could not be passed by reference»
(arbitro: variadic/by_ref_error.phpt), place (`$a[$k]`, `$o->p`) ⇒
MakeRef; (ii) fx21 guadagna una gamba vref DINAMICA (call_user_func) —
push_dyn_args probabilmente è già corretto e il fix non deve biforcare i
due sentieri; (iii) gate ORM/hk obbligatorio (cambio ref).

## (d) prima di (c): il corpus L'AVEVA GIÀ BECCATA — R-ST-107-4, KS-ST-107-1

**R-ST-107-4 — refuto la Scoperta 3 di S-105** («trovata dall'audit-fuga,
non da un test suite»): `Zend/tests/variadic/by_ref.phpt` (test($b,$c,$d)
= il caso ESATTO) e `by_ref_error.phpt` stanno nel corpus E nei 1417 fail
congelati (verificato in corpus-s105-{off,on}.fails). Il gate per NOME
guarda solo le REGRESSIONI: i rossi pre-esistenti sono invisibili come
scoperte. fx21 non ha trovato ciò che mancava; ha dissepolto ciò che il
congelamento copriva. **KS-ST-107-1**: ogni voce nuova a catalogo si cerca
PRIMA nel fail-set congelato; il fix §3.15 DEVE citare i fail da flippare
(attesa: 1417→1415, primo dividendo corpus dal congelamento).

## (c) Ordine fedeltà S-106 — A-ST-107-3

Riordino (costo/impatto): **1. §3.15** (perimetro minimo: un clamp + ramo
Error; classe «dato sbagliato silenzioso»; dividendo −2 fail nominati);
**2. generator get_gc** (resta la divergenza più GRAVE: massa illimitata
per-richiesta, arbitro = fixture rossa esistente); **3. §3.13 unit**;
**4. §3.12-i** DENTRO H-C3 (la fusione prop-RMW è il veicolo, A-ST-106-3
resta); **5. §3.14** fuori da ogni finestra leva (cura a due gradini già
vincolata, KS-MA-106-1 la recinta). Rispetto al mio WP-106: §3.15 non
esisteva; scavalca per costo, non per gravità.

Refutazioni capitali: NESSUNA.
