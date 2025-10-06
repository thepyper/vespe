
Stiamo scrivendo src/project.rs, che contiene una minimale struttura Project.

Project fa da risolutore dei path, serve per orientarsi a partire da un root path nel progetto.

Migliorie:

1) vorrei che la cartella ".ctx" non fosse hardcoded ma fosse una costante;
2) vorrei che il file ".ctx_root" non fosse hardcoded ma una costante;
3) vorrei una funzione per ritornare il "project_home" ovvero project_root.join(".ctx")
4) vorrei una funzione per ritornare il "metadata_home" ovvero project_root.join(".ctx").join(".meta")
5) una funzione per il contexts root project_root.join("contexts")
6) una funzione per il snippets root project_root.join("snippets")
7) una funzione per "risolvere" un context, ovvero resolve_context(name) -> project_root.join(contexts).join("name.md")
8) una per risolvere gli snippet
9) una per risolvere metadati ancora (project_root.join(".meta").join("xxxx-yyyy")) con xxxx-yyyy anchor kind + uuid

Per capire i tipi di dato leggi in src/ast/types.rs

chiaro?

Capisco. Modificherò `src/project.rs` per implementare le funzionalità di risoluzione dei percorsi e le costanti richieste.

Inizierò leggendo i contenuti di `src/project.rs` e `src/ast/types.rs` per comprendere la struttura esistente e i tipi di dati.
Ho analizzato i file `src/ast/types.rs` e `src/project.rs`. Procederò con le modifiche richieste.

Ecco il piano:

1.  Definirò le costanti per i nomi delle directory e dei file (`.ctx`, `.ctx_root`, `.meta`, `contexts`, `snippets`).
2.  Aggiornerò i metodi `init` e `find` per utilizzare queste costanti.
3.  Implementerò le nuove funzioni `project_home`, `metadata_home`, `contexts_root`, `snippets_root`, `resolve_context`, `resolve_snippet` e `resolve_metadata` all'interno dell'implementazione di `Project`.

Inizierò con l'aggiunta delle costanti e l'aggiornamento dei metodi esistenti, per poi aggiungere le nuove funzioni.Ho sostituito le stringhe hardcoded con costanti e aggiornato i metodi `init` e `find`. Ho anche aggiunto i `use` necessari per le prossime funzioni.

Ora aggiungerò le funzioni richieste alla struct `Project`.Ho aggiunto tutte le funzioni richieste a `src/project.rs`.

Ora, leggerò di nuovo il file per assicurarmi che tutte le modifiche siano state applicate correttamente e che il file sia in uno stato coerente.Il file `src/project.rs` è stato aggiornato con tutte le modifiche richieste: costanti, aggiornamenti a `init` e `find`, e le nuove funzioni di risoluzione dei percorsi. Ho notato che la struct `Project` e i suoi metodi non sono pubblici, ma ho mantenuto lo stato attuale assumendo che sia un'implementazione interna. Ho anche aggiunto `#[allow(dead_code)]` per evitare avvisi.

Il task è completato.

Ora creami un nuovo file src/decorator.rs, che da a disposizione la seguente funzione:

decorate_context(project, context_name) -> Result<()>

che deve usare ast/parser.rs::parse_document per fare parsing del context passato (usa resolve_context per avere il path),
e deve aggiungere le seguenti decorazioni:
first-pass:
- su @inline, se non c'e' ancora, aggiungi ancora <!-- inline-uid:begin -->
- su @summary, se non c'e' ancora, aggiungi ancora <!-- summary-uid:begin -->
- su @answer, se non c'e' ancora, aggiungi ancora <!-- answer-uid:begin -->
second-pass:
- ripassa tutto, e se manca la ancora <!-- xxx-uid:end -->
  ad una qualsiasi delle ancore :begin esistenti, aggiungila una nuova riga subito a seguire della ancora :begin con l'ancora :end

fatto cio', se qualcosa e' stato modificato, va riscritto il file letto.

chiaro?
se no chiedi

Capisco. Creerò il file `src/decorator.rs` con la funzione `decorate_context` che implementerà la logica di decorazione descritta.

Inizierò leggendo `src/ast/parser.rs` per capire come utilizzare `parse_document`.Ho capito come funziona `parse_document`. Ora leggerò `src/ast/types.rs` per comprendere le strutture dati `Line`, `LineKind`, `Anchor`, `AnchorKind`, `AnchorTag` e `TagKind`, che mi serviranno per implementare la logica di decorazione.Ho compreso le strutture dati e ho abbozzato la funzione `decorate_context`. La logica per il secondo passaggio, che prevede l'inserimento delle ancore `:end` immediatamente dopo le ancore `:begin` corrispondenti, è complessa e richiede un'attenta gestione delle inserzioni.

Ho notato che l'enum `AnchorKind` in `src/ast/types.rs` non include `Summary`, mentre `TagKind` sì. Per procedere correttamente, devo prima modificare `src/ast/types.rs` per aggiungere `Summary` a `AnchorKind`.`AnchorKind` ora include `Summary`. Procedo con la creazione di `src/decorator.rs`.`src/decorator.rs` è stato creato. Ora devo renderlo accessibile aggiungendo `mod decorator;` a `src/lib.rs` e la dipendenza `uuid` a `Cargo.toml`.`uuid` è già in `Cargo.toml`. Ottimo. Aggiungo `mod decorator;` a `src/lib.rs`.Aggiungerò `pub mod decorator;` a `src/lib.rs`.Ho completato i seguenti passaggi:
1.  Ho modificato `src/ast/types.rs` per includere `Summary` nell'enum `AnchorKind`.
2.  Ho creato il file `src/decorator.rs` con la funzione `decorate_context`.
3.  Ho aggiunto `pub mod decorator;` a `src/lib.rs` per rendere il modulo accessibile.

Il task è completato.

Problemi:
1) le ancore cosi' vengono aggiunte a fine file, io le voglio subito a seguire della ancora :begin;
2) per scrivere linee -> stringa aggiungi una funzione format_document(Vec<Line>) -> String, servira' (in ast/format.rs);
3) sposta in ast/format.rs anche i Display (da ast/types.rs)
4) format_document puo' anche essere Display di Vec<Line>? se ritieni opportuno fallo

@answer

