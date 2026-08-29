# Criterio S-164 p.1b — coppia dbal+ORM @ pin s163 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Harness: `s164-orm-coppia.sh` = COPIA DICHIARATA di s163-orm-coppia.sh
   (copia-gate a verifica POSITIVA dei path adattati + manifest
   s164-orm-copia.diff); pin s163 fea4a2d040a0d8d0 pena rc=9; i TRE
   adattamenti NEL CANONE restano nella copia: (i) rodaggio non giudicante;
   (ii) quiescenza per gamba (retry ×3); (iii) scaletta a DUE estremi.
   Fix LC_ALL=C, watchdog, pavimenti per-workspace MISURATI: EREDITATI.
2. RIF INVARIATO = registrato S-162 (NEXT_SESSION p.1: le 2 gambe s163
   [7,072;7,114] NON aggiornano il registrato): assoluto [34,47; 34,51],
   ORA_REF=4,885 (oracle net 4,87/4,90 dichiarate), rapporto net registrato
   S-162 [7,035; 7,086]; RES=0,293 (KS-146-1) INVARIATA.
3. ATTESA L-AU1 su ORM ~ZERO DICHIARATA: il criterio CLASSIFICA le forme —
   il fast path morde SOLO al MISS autoload [obj,metodo] k=1; Doctrine/Composer
   a REGIME risolvono da classmap (hit), i miss sono RARI e la quota miss NON
   è censita ⇒ tetto ~0 (nessuna componente censita da sommare). Esito atteso:
   COMPATIBILE dentro ±RES. ATTESA-AF1 resta APERTA per AMPIEZZA (aperture):
   questa coppia NON la risolve; le gambe si annotano per il pool (oggi 4).
4. COMPANION ASSOLUTO NOMINATO (rev. S-161 #3, VINCOLA LA LETTURA): la
   lettura DEVE citare il Delta assoluto phpr vs ref accanto al giudizio
   normalizzato; tensione (segni discordi) A VERBALE, mai taciuta.
5. SENTINELLA CONTAMINAZIONE (lezione S-161): oracle net fuori dal SUO
   riferimento storico (4,87/4,90 ± dintorni RES) ⇒ finestra sospetta ⇒
   cifra NULLA anche a flag per-gamba muti; parità resta valida.
6. Parità: ORM per NOME vs baseline 16; dbal fail-set phpr STABILE tra gambe
   (10 nomi attesi vs oracle, latin1 a catalogo; riserva ictx-oracle
   RICORRENTE s162+s163 a verbale: le gambe ictx si annotano per
   l'istruttoria in agenda p.5). Parità rotta ⇒ CIFRA NULLA.
7. Esiti a FILE: verdetto `s164-orm-coppia-verdetto.out`; rc SOLO da
   orm-out/rimisura.done; catena: lancio SOLO a pair164-t14.done rc=0
   (s164-lancio-orm.sh); MAPPA_SP dedicato APFS /private/tmp/phpr-s164-orm.
   Sentinelle LS inizio/fine; lock della SESSIONE solo VERIFICATO.
