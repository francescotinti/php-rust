# Criterio S-152 p.2b — rerun QUIET della testa hostcall (rev. az.3 S-151; pre-registrato PRIMA del run)

1. Oggetto: il Δ313 (hostcall_n r1=82.211.375 vs r2=82.211.688, s151 sotto
   CI busy) — meccanismo da nominare o repliche quiet. CONTEGGI, mai tempo.
2. Probe = census s151 CONSERVATO ab02faec0abfab67 (stesso binario, stessa
   ricetta §4 del criterio s151); stesso workload ORM (tarball gates);
   workdir path ≥100 char come s151; 2 repliche SEQUENZIALI con lock di
   finestra + quiescenza s129 in retry PRIMA delle repliche (differenza
   UNICA rispetto a s151: sentinelle busy attese CLEAN).
3. Giudizio pre-registrato: |Δ hostcall_n r1−r2| = 0 ⇒ il non-determinismo
   di s151 è ATTRIBUITO alla contesa (meccanismo indiretto, dichiarato) e la
   testa hostcall (82,2M, per-NOME) diventa CITABILE nei criteri con la
   cifra delle repliche quiet; Δ ≠ 0 ⇒ testa NON citabile, meccanismo resta
   da nominare (apertura per NOME), i canali C1–C5 restano validi (identità
   §3 s151 già rc=0, lo scarto riguarda la sola testa).
4. Sentinelle non-gate dichiarate: fail-set r1==r2==baseline16; busy
   pre/post per replica; rc phpunit atteso 2 (fail-set noto).
