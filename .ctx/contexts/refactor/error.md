@include rules

# refactor/error - error

Ora facciamo un refactoring grosso, ma a piccoli bocconi.
Si tratta di modificare in tutta la libreria src/* (non in bin/) la gestione degli errori.

1) Definisci in ogni modulo un errore specifico, che comprenda tutti gli errori da gestire nel modulo (che ora sono gestiti un po a casaccio, spesso con anyhow); NESSUN CAMBIO DI LOGICA DI NESSUN TIPO, SOLO CAMBIO DEL TIPO DI ERRORE;
2) Definisci in src/error.rs un errore globale, che comprenda una conversione da tutti gli errori specifici usando thiserror e la sua magica macro #from. NESSUN CAMBIO DI LOGICA DI NESSUN TIPO, SOLO CAMBIO DEL TIPO DI ERRORE;

chiaro?

