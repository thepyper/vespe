@include project
@include agent/planner

Voglio aggiungere in src/execute/mod.rs un oggetto in ExecuteWorker, ovvero

variables: HashMap<String, String>   // insieme dei valori attuali delle variabili

Queste variabili possono variare durante il parsing del file context, ogni volta che viene incontrato un tag @set[variable=value].

Il tag @set va aggiunto alla gestione correttamente in src/syntax e in src/semantic.

Pianifica questo cambiamento, senza eseguirlo.

<!-- answer-6213b6ce-76eb-452a-9f6e-06dccbdbafc4:begin -->
Comprendo la tua richiesta. Aggiungerò un campo `variables: HashMap<String, String>` all'oggetto `ExecuteWorker` in `src/execute/mod.rs` e implementerò la gestione del nuovo tag `@set[variable=value]` in `src/syntax` e `src/semantic`.

Ecco il piano dettagliato:

1.  **Analisi del codice esistente:**
    *   Leggerò `src/execute/mod.rs` per identificare la posizione e la struttura dell'oggetto `ExecuteWorker`.
    *   Esaminerò `src/syntax/mod.rs`, `src/syntax/parser.rs` e `src/syntax/types.rs` per capire come vengono definiti e parsificati i tag esistenti.
    *   Controllerò `src/semantic/mod.rs` e `src/semantic/context.rs` per comprendere come i tag vengono elaborati semanticamente.

2.  **Modifica di `src/execute/mod.rs`:**
    *   Aggiungerò il campo `variables: HashMap<String, String>` all'oggetto `ExecuteWorker`.
    *   Inizializzerò questa `HashMap` come vuota durante la creazione di `ExecuteWorker`.

3.  **Modifica di `src/syntax`:**
    *   In `src/syntax/types.rs`, aggiungerò una nuova variante `Set { variable: String, value: String }` all'enum `Tag` (o struttura equivalente che definisce i tag).
    *   In `src/syntax/parser.rs`, modificherò la logica di parsing per riconoscere il pattern `@set[variable=value]`. Questo richiederà l'estrazione della chiave (`variable`) e del valore (`value`) all'interno delle parentesi quadre.

4.  **Modifica di `src/semantic`:**
    *   In `src/semantic/context.rs` (o file correlato che gestisce l'elaborazione dei tag), aggiornerò la logica per gestire il nuovo tag `Set`.
    *   Quando viene incontrato un tag `Set`, la logica dovrà estrarre la variabile e il valore e propagarli in modo che `ExecuteWorker` possa utilizzarli.

5.  **Implementazione della logica di `@set` in `ExecuteWorker`:**
    *   Nel metodo di esecuzione di `ExecuteWorker` (o funzione correlata), aggiungerò la logica per iterare sui tag parsificati.
    *   Quando viene rilevato un tag `Set`, aggiornerò la `HashMap` `variables` con la chiave e il valore estratti.

6.  **Test e verifica:**
    *   Scriverò nuovi unit test per `src/syntax/parser.rs` per assicurare che il tag `@set` venga parsificato correttamente.
    *   Aggiungerò unit test per `src/semantic/context.rs` per verificare che il tag `Set` venga elaborato semanticamente in modo appropriato.
    *   Creerò test di integrazione per `ExecuteWorker` per confermare che la `HashMap` `variables` venga aggiornata correttamente quando un file di contesto contiene tag `@set`.
    *   Eseguirò tutti i test esistenti per garantire che le modifiche non abbiano introdotto regressioni.
    *   Eseguirò il linter e il formatter del progetto per mantenere la coerenza dello stile del codice.

Procederò con l'analisi del codice esistente.
<!-- answer-6213b6ce-76eb-452a-9f6e-06dccbdbafc4:end -->