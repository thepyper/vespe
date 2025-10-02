@include ctx

# prompt

Voglio aggiungere la direttiva @prompt.

A livello di funzionalita', e' solo sintactic sugar: 

//@prompt <context>

equivale a

//@include <context>
//@answer

Pero' introduce una distinzione interessante: i <context> che voglio includere in questo modo
molto probabilmente sono degli snippet, che a loro volta NON vanno processati dalla direttiva watch;

perche' uno snippet potrebbe contenere diverse direttive @answer, ad esempio.
!!!!!
anzi, ecco!

in realta' il pattern non e' @prompt <context>

ma e' @include <snippet>

Voglio quindi poter scrivere:

"@include snippets/do_antipattern_search"

Se funziona, e watch decidiamo che watcha solo la directory principale, gia' e' come voglio.
Oppure, una estensione diversa forse potrebbe funzionare ancora meglio?

do_antipattern_search.sn ?

Dimmi che pensi di questi pensieri, ragiona bene e proponi.

Comprendo. L'obiettivo è introdurre una direttiva per includere "snippet" che non dovrebbero essere processati dalla direttiva `watch`, e si sta valutando se `@prompt <context>` o `@include <snippet>` sia la strada migliore, magari con un'estensione `.sn`.

Per prima cosa, esaminerò il `Cargo.toml` e i file sorgente in `src/bin/ctx/` per capire come funziona attualmente il processamento dei contesti e la direttiva `watch`.

Ho analizzato il `Cargo.toml` e la struttura dei file in `src/bin/ctx`. Il crate `notify` è un chiaro indicatore della funzionalità di `watch`.

Concordo che l'introduzione di un meccanismo per includere file (snippet) che non vengono "watched" è una buona idea.

**Riepilogo della mia analisi e proposta:**

1.  **Direttiva:** L'approccio `@include <snippet_file.sn>` è più diretto e generico rispetto a `@prompt <context>`, e si allinea meglio con la tua chiarificazione.
2.  **Identificazione Snippet:** L'uso di una nuova estensione `.sn` (per snippet) è il metodo più pulito ed efficiente per distinguere i file che non devono essere monitorati dal `watch`er.
3.  **Comportamento `watch`:** Il meccanismo di `watch` (probabilmente in `main.rs` o `project.rs` che usa `notify`) verrà modificato per ignorare i file con estensione `.sn`.
4.  **`@answer` negli snippet:** Per ora, assumerò che `@answer` all'interno di un file `.sn` si comporti come in qualsiasi altro file di contesto, delimitando la parte di prompt e risposta. Se desideri un comportamento diverso, fammelo sapere.

**Piano di implementazione:**

1.  Identificare i file responsabili del parsing delle direttive (`ast.rs`, `composer.rs`).
2.  Identificare i file responsabili della configurazione del file watcher (`main.rs`, `project.rs`).
3.  Modificare il file watcher per ignorare i file `.sn`.
4.  Assicurarsi che la logica di inclusione legga correttamente i file `.sn`.

**Domande per te prima di procedere:**

1.  Confermi l'uso dell'estensione `.sn` per i file snippet e la modifica del `watch`er per ignorarli?
2.  Il comportamento standard di `@answer` all'interno di un file `.sn` è accettabile, o hai in mente un comportamento specifico per questa direttiva negli snippet?
3.  Hai preferenze sulla posizione dei file `.sn` (es. in una directory `snippets/` dedicata o possono essere ovunque)?


Ora mi sono chiarito la distinzione, sono due:
1) gli snippet non sono interessati da watch
2) gli snippet quando sono inclusi vanno trattati come parte del context includente, lo snippet non va modificato!

Ad esempio, se nel context pensierini.md includo lo snippet fai_qualcosa.sn e in fai_qualcosa.sn c'e' un @answer, la modifica del
file deve avvenire in pensierini.md, NON in fai_qualcosa.sn; praticamente lo snippet va "copiaincollato" dentro al file md.

Alternativa, potrei usare una direttiva piu' specifica per gli snippet, e piu' indicativa come @inline <nomesnippet>;
questo mi permette di validare che sia uno snippet (non posso fare @inline di un .md).

Che ne pensi?
Leggi i sorgenti per avere del contesto prima di rispondere.

Comprendo le tue nuove specifiche per la direttiva `@inline`. La distinzione tra "non watched" e "copy-pasted" è cruciale e cambia significativamente l'approccio rispetto a un semplice `@include`.

Prima di rispondere, leggerò i sorgenti in `src/bin/ctx/` e `Cargo.toml` per capire l'attuale implementazione del parsing delle direttive, la composizione dei contesti e il funzionamento del `watch`er. Questo mi permetterà di valutare al meglio le implicazioni delle tue richieste e di proporti una soluzione robusta.

Ho la lista dei file in `src/bin/ctx/`. Ora leggerò `Cargo.toml` per capire le dipendenze e la struttura del progetto. Successivamente, esaminerò `ast.rs`, `composer.rs`, `main.rs` e `project.rs` per comprendere l'implementazione attuale delle direttive e del monitoraggio dei file.

Ho esaminato `Cargo.toml`. Le dipendenze chiave per questo compito sono `notify` (per il monitoraggio dei file), `markdown` (probabilmente per l'analisi dei file markdown e delle direttive), `anyhow` e `thiserror` (per la gestione degli errori) e `clap` (per l'analisi degli argomenti CLI).

Ora, esaminerò i file sorgente in `src/bin/ctx/` per comprendere l'implementazione attuale. Inizierò con `main.rs` per vedere il punto di ingresso e come viene avviato il `watch`, poi `project.rs` per la logica a livello di progetto, `ast.rs` per l'Abstract Syntax Tree (AST) e l'analisi delle direttive, e infine `composer.rs` per come vengono composti i contesti.

Leggerò prima `main.rs`.

Ho esaminato `main.rs`. Ecco i punti chiave:

*   **Comando `watch`:** Il comando `Watch` utilizza la crate `notify` per monitorare ricorsivamente `project.contexts_dir()`. Filtra specificamente i file con estensione `.md` (`path.extension().map_or(false, |ext| ext == "md")`). Qui dovremo modificare la logica per ignorare i file `.sn`.
*   **Esecuzione del contesto:** Quando viene rilevata una modifica in un file `.md`, viene chiamato `project.execute_context()`.
*   **Struttura del progetto:** Importa i moduli `project`, `agent_call`, `ast` e `composer`.

Ora, esaminiamo `project.rs` per capire come vengono gestiti i contesti e come viene determinato `contexts_dir()`. Questo sarà importante per definire dove possono risiedere i file `.sn`.

Ho esaminato `project.rs`. Ecco i risultati chiave:

*   **`contexts_dir()`:** Questa funzione restituisce `self.root_path.join("contexts")`. Ciò significa che tutti i file di contesto (attualmente file `.md`) dovrebbero trovarsi in una sottodirectory `contexts` all'interno della directory `.ctx` del progetto.
*   **`list_contexts()`:** Questa funzione scorre `contexts_dir()` e filtra i file con estensione `.md`. Questo è un altro punto in cui dovremo adattare la logica per includere o escludere i file `.sn` in base al comportamento di `watch`.
*   **`compose()` e `context_tree()`:** Queste funzioni si basano su `get_or_build_ast()`, che a sua volta chiama `crate::ast::ContextAstNode::build_context_ast()`. Ciò suggerisce che `ast.rs` è responsabile dell'analisi delle direttive e della costruzione dell'AST.
*   **`execute_context()`:** Qui viene gestita la direttiva `@answer`. Componi il contesto, trova la riga `@answer`, chiama l'LLM e quindi *sostituisce la riga `@answer` nel file sorgente originale* con la risposta dell'LLM. Questa è un'informazione fondamentale per il comportamento "copia-incolla" di `@inline`.

Ora, esaminiamo `ast.rs` per capire come vengono analizzate le direttive come `@include` e `@answer` e come viene costruito l'AST. Questo sarà cruciale per implementare la nuova direttiva `@inline` e il suo comportamento "copia-incolla".

Ho esaminato `ast.rs`, che è fondamentale per l'analisi delle direttive e la costruzione dell'AST.

Punti chiave:

*   **`LineData` enum:** Definisce le direttive supportate: `Include`, `Answer`, `Summary`, `Text`. Dovremo aggiungere una variante per `@inline`.
*   **`ContextAstNode::parse()`:** Analizza le direttive basandosi sui prefissi delle stringhe. Qui aggiungeremo la logica per `@inline`.
*   **`ContextAstNode::build_context_ast()`:** Costruisce ricorsivamente l'AST leggendo i contesti inclusi e usando `resolve_context_path()` per trovare i percorsi dei file.
*   **`resolve_context_path()` e `to_filename()`:** Attualmente assumono che tutti i file di contesto terminino con `.md`. Dovranno essere aggiornati per gestire i file `.sn`.
*   **`Line` struct:** Memorizza `LineData`, `source_file` e `source_line_number`, essenziali per il comportamento "copia-incolla" di `@answer` in `project.rs`.

**Riepilogo della comprensione attuale:**

*   **Meccanismo `watch`:** In `main.rs`, il comando `watch` filtra i file `.md`. Deve essere aggiornato per ignorare i file `.sn`.
*   **`@include`:** Include file `.md` e costruisce ricorsivamente l'AST.
*   **`@answer`:** In `project.rs`, `execute_context` trova il tag `@answer` nel contesto composto e lo sostituisce nel *file sorgente originale*.

**Piano proposto per `@inline`:**

1.  **Definire l'estensione `.sn`:** Confermare con l'utente l'estensione desiderata per gli snippet.
2.  **Modificare `main.rs` (watch):** Aggiornare il comando `watch` per ignorare i file `.sn`.
3.  **Modificare `ast.rs` (parsing):**
    *   Aggiungere una nuova variante `LineData::Inline { snippet_name: String }`.
    *   Modificare `ContextAstNode::parse()` per riconoscere `@inline <snippet_name>`.
    *   Aggiornare `resolve_context_path()` e `to_filename()` per gestire i file `.sn`.
4.  **Modificare `composer.rs` (composizione):** Implementare il comportamento "copia-incolla". Quando `ContextComposer` incontra `LineData::Inline`, dovrà:
    *   Leggere il contenuto del file snippet (`snippet_name.sn`).
    *   Analizzare il contenuto dello snippet in `Line`s.
    *   Inserire queste `Line`s nel contesto composto, *senza modificare i loro `source_file` e `source_line_number`*. Ciò garantisce che un `@answer` all'interno di uno snippet inlined venga scritto nel *file originale che conteneva la direttiva `@inline`*.
5.  **Modificare `project.rs` (`list_contexts`):** Aggiornare `list_contexts` per escludere i file `.sn` dall'elenco dei contesti di primo livello.

**Domande per l'utente:**

1.  Confermi che `.sn` è l'estensione desiderata per i file snippet?
2.  I file `.sn` devono essere elencati quando si esegue `ctx list`? La mia idea attuale è no, poiché sono destinati a essere inlined e non contesti di primo livello.
3.  Hai requisiti specifici per la gestione degli errori se una direttiva `@inline` punta a un file snippet inesistente o a un file che non è un `.sn`?

Ora leggerò `composer.rs` per capire come l'AST viene attraversato e le righe vengono composte, il che informerà l'implementazione del passaggio 4 del piano.

Ho esaminato `composer.rs`, che è il punto in cui l'AST viene attraversato e il contesto finale composto viene generato.

Ecco cosa ho raccolto:

*   **`compose_from_ast()`:** Questa è la funzione principale. Itera attraverso le `lines` di un `ast_node`.
*   **Gestione di `LineData::Include`:** Quando viene incontrata una direttiva `Include`, chiama ricorsivamente `compose_from_ast()` sul nodo figlio corrispondente nell'AST ed estende le `composed_lines` con il risultato. Ciò significa che il contenuto incluso viene completamente elaborato e appiattito nel contesto padre.
*   **Gestione di `LineData::Summary`:** Chiama `self.project._handle_summary_tag()` per ottenere il testo riassunto e quindi lo aggiunge come `LineData::Text` alle `composed_lines`. Fondamentalmente, preserva `source_file` e `source_line_number` dalla direttiva `@summary` originale.
*   **Caso predefinito (`_`):** Per qualsiasi altro `LineData` (come `Text` o `Answer`), clona semplicemente la riga e la aggiunge a `composed_lines`.

**Implicazioni per `@inline`:**

L'attuale struttura di `compose_from_ast()` è adatta per implementare il comportamento "copia-incolla" di `@inline`. Quando incontriamo un `LineData::Inline`, dovremo:

1.  Risolvere il percorso del file snippet.
2.  Leggere il contenuto del file snippet.
3.  Analizzare il contenuto del file snippet in un `Vec<Line>`.
4.  Iterare attraverso queste righe di snippet e aggiungerle a `composed_lines`.
5.  **Fondamentalmente:** Per ogni riga dello snippet, *sovrascriveremo* il suo `source_file` e `source_line_number` con quelli della *direttiva `@inline` originale*. Ciò garantisce che se un `@answer` è presente nello snippet, la logica di `project.rs` scriverà la risposta LLM nel file che conteneva la direttiva `@inline`, non nel file snippet stesso.

**Piano rivisto (incorporando le intuizioni di `composer.rs`):**

1.  **Confermare l'estensione `.sn`:** Attendere la conferma dell'utente.
2.  **Modificare `main.rs` (watch):** Aggiornare il comando `watch` per ignorare i file con estensione `.sn`.
3.  **Modificare `ast.rs` (parsing):**
    *   Aggiungere una nuova variante `LineData::Inline { snippet_name: String }`.
    *   Modificare `ContextAstNode::parse()` per riconoscere `@inline <snippet_name>` e creare voci `LineData::Inline`.
    *   Aggiornare `resolve_context_path()` e `to_filename()` per gestire i file `.sn`. Ciò comporterà la creazione di una nuova `resolve_snippet_path` e `to_snippet_filename` o la modifica di quelle esistenti per essere più generiche.
4.  **Modificare `composer.rs` (composizione):**
    *   In `compose_from_ast()`, aggiungere un braccio `match` per `LineData::Inline`.
    *   All'interno di questo braccio:
        *   Risolvere il percorso del file snippet utilizzando una nuova funzione helper (ad esempio, `self.project.resolve_snippet_path()`).
        *   Leggere il contenuto del file snippet.
        *   Analizzare il contenuto dello snippet in `Vec<Line>` utilizzando `ContextAstNode::parse()`.
        *   Per ogni `line` nelle righe dello snippet analizzate, creare una nuova struct `Line` in cui `data` viene clonata dalla riga dello snippet, ma `source_file` e `source_line_number` vengono presi dalla *`line` della direttiva `@inline` originale*.
        *   Estendere `composed_lines` con queste righe di snippet modificate.
5.  **Modificare `project.rs` (`list_contexts`):** Aggiornare `list_contexts` per filtrare i file `.sn`.

**Domande per l'utente (ribadendo e aggiungendo dettagli):**

1.  **Conferma dell'estensione `.sn`:** Confermi che `.sn` è l'estensione desiderata per i file snippet?
2.  **Comportamento di `ctx list`:** I file con estensione `.sn` devono essere elencati quando si esegue `ctx list`? La mia forte raccomandazione è *no*, poiché sono destinati a essere inlined e non trattati come contesti indipendenti di primo livello.
3.  **Gestione degli errori per snippet inesistenti:** Cosa dovrebbe succedere se una direttiva `@inline` punta a un file snippet che non esiste? Dovrebbe:
    *   Causare un errore irreversibile e interrompere l'esecuzione?
    *   Registrare un avviso e saltare l'inline?
    *   Registrare un avviso e inserire un testo segnaposto?
4.  **Posizione dei file snippet:** I file snippet (`.sn`) possono essere posizionati ovunque all'interno del progetto, o dovrebbero essere limitati a una directory specifica (ad esempio, `contexts/snippets/` o `snippets/` nella root del progetto)? La mia raccomandazione è di posizionarli in una directory `snippets` dedicata all'interno della cartella `.ctx` (ad esempio, `.ctx/snippets/`) per mantenerli organizzati e separati dai contesti principali.

Una volta che avrò le tue risposte, potrò procedere con l'implementazione.

Vediamo, raffino un po il piano:

1) Separiamo gli snippet in una seconda cartella, "snippets"; quindi implementare la logica similmente a contexts_dir() per gli snippet; list_snippets() eccetera; 
2) Confermo l'estensione ".sn", tuttavia mettiamola in una costante; anche l'estensione dei contesti ".md" mettiamola in una constante non hardwired;
3) Confermo la direttiva @inline che puo' includere uno snippet con effetto "copia ed incolla", non modificando realmente il file context in cui e' inlined, ma a livello di gestione AST e generazione del contesto; l'inline si deve comportare incollando le righe dello snippet nella AST (o comunque facendole comparire nella AST opportunamente); forse per implementare questa cosa sarebbe prima preferibile implementare un visitor o un iteratore sulla AST, per avere una astrazione piu' pulita? valutare!!
4) Nessun problema con watch, tanto la cartella snippets separata risolve alla radice questo problema.

Riformula il piano alla luce di queste info

Ottimo, il piano è molto più chiaro ora. Ho incorporato le tue ultime indicazioni.

**Riepilogo del Piano Aggiornato:**

**Fase 1: Struttura del Progetto e Costanti**

1.  **Definire Costanti per le Estensioni:**
    *   Introdurre costanti per l'estensione dei contesti (`.md`) e degli snippet (`.sn`) in una posizione appropriata (es. `project.rs` o un nuovo modulo `constants.rs`).
2.  **Implementare `snippets_dir()` in `project.rs`:**
    *   Aggiungere una nuova funzione `snippets_dir()` a `project.rs` che restituisce il percorso assoluto alla directory degli snippet.
3.  **Implementare `list_snippets()` in `project.rs`:**
    *   Aggiungere una funzione `list_snippets()` simile a `list_contexts()` che elenca i file con estensione `.sn` all'interno di `snippets_dir()`.

**Fase 2: AST e Parsing delle Direttive (`ast.rs`)**

1.  **Aggiungere la variante `LineData::Inline`:**
    *   Modificare l'enum `LineData` in `ast.rs` per includere `Inline { snippet_name: String }`.
2.  **Aggiornare `ContextAstNode::parse()`:**
    *   Aggiungere la logica a `ContextAstNode::parse()` per riconoscere la direttiva ` @inline <snippet_name>`.
    *   Quando viene trovata ` @inline`, creare una variante `LineData::Inline` con il `snippet_name` estratto.
3.  **Aggiornare la Risoluzione dei Percorsi:**
    *   Modificare `resolve_context_path()` e `to_filename()` (o creare nuove funzioni helper come `resolve_snippet_path` e `to_snippet_filename`) per localizzare correttamente i file `.sn` all'interno di `snippets_dir()`. Questo sarà utilizzato quando `composer.rs` dovrà leggere il contenuto dello snippet.  ! direi creare nuove funzioni, non intasare le esistenti; context e snippet ben distinti !

**Fase 3: Composizione del Contesto (`composer.rs`)**

1.  **Implementare la gestione di ` @inline` in `compose_from_ast()`:**
    *   Aggiungere un nuovo `match` arm per `LineData::Inline` in `compose_from_ast()`.
    *   All'interno di questo arm:
        *   Ottenere il `snippet_name` da `LineData::Inline`.
        *   Risolvere il percorso completo del file snippet utilizzando le nuove funzioni di `project.rs`.
        *   Leggere il contenuto del file snippet.
        *   Parsare il contenuto dello snippet in un `Vec<Line>` utilizzando `ContextAstNode::parse()` (o un meccanismo di parsing simile).
        *   **Fondamentale:** Per ogni `Line` parsata dallo snippet, creare un *nuovo* oggetto `Line`. Il campo `data` di questa nuova `Line` sarà clonato dalla riga dello snippet. Tuttavia, `source_file` e `source_line_number` di questa nuova `Line` saranno impostati su quelli della *riga della direttiva ` @inline` originale*. Questo assicura che qualsiasi ` @answer` all'interno dello snippet inlined causerà modifiche al *file di contesto padre*, non al file snippet stesso.
        *   Estendere `composed_lines` con questi oggetti `Line` appena costruiti e modificati.
2.  **Valutazione AST Visitor/Iterator:**
    *   Per ora, procederò con la modifica diretta di `compose_from_ast()`. Valuterò l'introduzione di un pattern visitor/iterator se la complessità dovesse richiederlo durante l'implementazione.

**Fase 4: Comportamento `watch` e `list` (`main.rs`, `project.rs`)**

1.  **`watch` in `main.rs`:**
    *   Nessuna modifica necessaria, poiché `watch` monitora solo `project.contexts_dir()`, e gli snippet saranno in una `snippets_dir()` separata.
2.  **`list_contexts()` in `project.rs`:**
    *   Assicurarsi che `list_contexts()` continui a elencare solo i file `.md` da `contexts_dir()`.
3.  **`list_snippets()` in `project.rs`:**
    *   Questa nuova funzione elencherà i file `.sn` da `snippets_dir()`.

**Gestione degli Errori:**

*   **File Snippet Mancante:** Se una direttiva ` @inline` punta a un file `.sn` inesistente, il sistema dovrebbe restituire un errore irreversibile durante la composizione, impedendo la generazione di un contesto incompleto o errato.

**Domande per te prima di procedere:**

1.  **Posizione della directory `snippets`:** La directory `snippets` dovrebbe essere `c:\Proj\xxx\025_006.Vespe\vespe\.ctx\snippets` (cioè, una sottodirectory di `.ctx`) o `c:\Proj\xxx\025_006.Vespe\vespe\snippets` (cioè, direttamente sotto la root del progetto)? La mia raccomandazione è `.ctx/snippets` per mantenere l'organizzazione.
2.  **Gestione degli errori per snippet mancanti:** Confermi che un errore irreversibile (che blocca l'esecuzione) è il comportamento desiderato se un file snippet referenziato da ` @inline` non esiste?
3.  **AST Visitor/Iterator:** Confermi che possiamo procedere con la modifica diretta di `compose_from_ast()` per ora, e valuteremo un pattern visitor/iterator solo se la complessità lo richiederà?

Attendo le tue risposte per procedere con l'implementazione.

1. La cartella snippets allo stesso livello di contexts. quindi si, .ctx/snippets
2. si, come un include mancante
3. va bene

Procedi pure con implementazione.
Fai commit granulari.

---
Ok vedo che hai praticamente finito.

Intanto fai commit.

@answer

Poi c'e' un problemino con watch:
pare non rilevare il fatto che test_inline.md sia eseguibile, nonostante contenga (indirettamente) una @answer

@answer

