# S-142 — verdetto sonda profondità nesting (az.rev. S-141 #3)

Sonda: `probe-nesting.php` (costruzione ITERATIVA, ricorsione solo nel drop;
argv = profondità, repr p/h). Binari: A = stash `phpr-s140` (f2708b75) ·
B = build fresca A′ (bba8a734, gemello candidato). Sweep 8k→256k ×2, poi
bisezione a risoluzione 2k.

| binario | repr | ultimo OK | sfonda |
|---|---|---|---|
| A | packed | ~74.000 | ~76.000 (rc=134) |
| A | hashed | ~74.000 | ~76.000 (rc=134) |
| B | packed | ~74.000 | ~76.000 (rc=134) |
| B | hashed | ~74.000 | ~76.000 (rc=134) |

**Soglia INVARIATA A vs B alla risoluzione 2k, entrambe le repr** — il timore
del revisore (frame per livello cambiati dal drop() a 204 istr ⇒ soglia mossa)
è RIENTRATO. Lo sfondamento è rc=134 (SIGABRT dello stack-guard Rust, abort
pulito), non SIGSEGV; margine ampio sul riferimento WP-25 (~45k). Il claim del
criterio «profondità di ricorsione INVARIATA» è ora MISURATO, non solo argomentato.
