# s126-criterio-orm.md — istruttoria ORM 8,5× (REGISTRAZIONE, non leva) — PRE-registrato

1. Oggetto: prezzare BILATERALMENTE i tre indiziati del gap ORM (PERF_MAP): (a) compile-per-classe via eval (sentiero mock PHPUnit) → micro `evalcls`; (b) reflection → micro `refl`; (c) churn oggetti/identity-map (UoW) → micro `objchurn`. Sorgenti in `wp126-harness/micro-orm/`, STESSO file sui due motori.
2. Metodo = run-micro.sh (spina dorsale S-97): user CPU `/usr/bin/time -p`, R=5 alternato (oracle,phpr per ripetizione), pavimento PER-BINARIO su `empty.php` (mediana R=5), N emesso dal sorgente a ogni run, mediana E spread pubblicati.
3. Parità: stdout dei due motori IDENTICO per categoria (somme deterministiche), pena cifra NULLA sulla categoria.
4. Arbitro `s126-orm-micro.sh` committato in QUESTO commit; cifre citabili solo da `s126-orm-micro-verdetto.out`. Alarm 900 s per invocazione (alarm sopravvive all'exec): scaduto ⇒ categoria NULLA DICHIARATA, non hang.
5. Indizio co-registrato (UNILATERALE, mai cifra): 2 campioni `sample` da 25 s del phpr in run ORM reale (`s126-orm-profile.sh`, stesso commit), sezione top-of-stack a verbale; serve a PESARE le categorie dentro il run reale, non a firmare attribuzioni (REGOLE §4).
6. Lettura pre-registrata: la leva si NOMINA sulla categoria con rapporto massimo se >6 (sopra il tetto delle micro esistenti, prop 5,6) E visibile nel profilo; se nessuna supera 6 ⇒ verdetto «indiziati non confermati», istruttoria da estendere (subset della suite per gruppo) — NESSUNA leva di ripiego.
7. Nessuna predizione di magnitudine (prima misura di queste categorie). Perimetro esteso ⇒ ri-commit del criterio PRIMA del rerun.
