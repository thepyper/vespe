@include project.md

# Mvp

Stiamo lavorando in src/bin/ctx al binario ctx che e' il tool principale che stiamo costruendo adesso.

Stiamo producendo un mvp del progetto, abbiamo gia' un mini-prototipo funzionante, che sta producendo i contesti che vedi.
Abbiamo fatto un discreto refactoring spostando le funzionalita' in Project e Context, invece di avere tutto su main.

Prompt:
La prossima modifica richiede un po' di lavoro, si tratta di internalizzare la chiamata alla cli degli llm.
Mentre fino ad ora l'utilizzo del tool e' tipo:

ctx compose mvp | gemini -p -y -m gemini-2.5-flash

Poi dovrebbe diventare 

ctx execute mvp 

Che deve occuparsi di chiamare il llm opportuno (per ora sempre gemini -p -y -m gemini-2.5-flash), e processare i dati di ritorno.
Quindi creare il piping opportuno.

Per ora aggiungiamo il comando execute che esegue il context. I dati di ritorno teniamoli, ma per ora non li usiamo. 
Serviranno per implementare poi @answer.

Esegui pure questa modifica se non hai dubbi.

Response:
Capisco. L'obiettivo è internalizzare la chiamata all'LLM all'interno del comando `ctx`, passando da un piping esterno (`ctx compose mvp | gemini ...`) a un comando unificato (`ctx execute mvp`). Il comando `execute` dovrà chiamare `ctx compose`, prendere il suo output, passarlo a `gemini -p -y -m gemini-2.5-flash`, e catturare l'output dell'LLM per un uso futuro (come `@answer`).

Ecco il mio piano:

1.  **Analisi della struttura esistente:** Esaminerò i file in `src/bin/ctx` per capire come sono definiti e gestiti i comandi attuali (in particolare `compose`).
2.  **Definizione del comando `execute`:** Aggiungerò una nuova sottocomando `execute` al parser degli argomenti CLI, che accetterà il nome del contesto come parametro.
3.  **Implementazione della logica di `execute`:**
    *   All'interno del gestore del comando `execute`, chiamerò internamente `ctx compose <nome_contesto>` per ottenere il contenuto del contesto.
    *   Catturerò l'output di `ctx compose`.
    *   Eseguirò il comando LLM (`gemini -p -y -m gemini-2.5-flash`) passando l'output catturato come input.
    *   Catturerò l'output del comando LLM.
    *   Per ora, mi limiterò a stampare l'output dell'LLM o a memorizzarlo in una variabile, senza ulteriori elaborazioni.

Inizierò listando i file in `src/bin/ctx` per avere una chiara visione della struttura attuale.`src/bin/ctx` ha `context.rs`, `main.rs`, `project.rs`. `main.rs` gestisce il parsing CLI e il dispatch dei comandi. `context.rs` e `project.rs` contengono la logica di contesto e progetto. Leggerò `main.rs` per capire la definizione dei comandi e la gestione di `compose`.Ho analizzato `main.rs`: `clap` gestisce gli argomenti CLI, `Cli` e `Commands` definiscono i sottocomandi, `Compose` usa `project.compose` e stampa l'output. `Project` gestisce i contesti.

Il piano per il comando `execute` è:
1.  Aggiungere `Execute { name: String }` all'enum `Commands` in `main.rs`.
2.  Implementare la logica `execute` nella funzione `main`:
    *   Chiamare `project.compose(&name)?` per il contenuto del contesto.
    *   Costruire il comando `gemini -p -y -m gemini-2.5-flash`.
    *   Usare `std::process::Command` per eseguire `gemini`, reindirizzando l'output di `compose` all'input di `gemini`.
    *   Catturare e stampare l'output e l'errore standard di `gemini`.
    *   Aggiungere `use std::process::{Command, Stdio};` a `main.rs`.

Confermo il piano con l'utente.

Prompt:
Approvo il piano, esegui.

Prompt:
Ok, ora specifichiamo meglio il comando execute.
Innanzitutto, va spostato in Project::execute_context.
Il comando funziona cosi, pseudocodice:

while (is_context_executable) {
    execute_context_step
}

Ora, cosa significa che il context e' eseguibile?
Significa che, nel parsing del context, trovo un tag eseguibile.
Ad esempio @answer.

Se c'e' un tag eseguibile, compose_recursive dovrebbe fermarsi, e ritornare il tag eseguibile.

execute_context_step dovrebbe eseguire il contesto ricevuto (ovvero la composizione, tagliata fino al tag trovato)
e inserire la risposta al posto del tag (che va quindi rimosso). Quindi, riscrivere il file .md che sta venendo eseguito.

A quel punto, il loop continua fino a che non ci sono piu' tag eseguibili.
E' chiaro il punto? Se si illustra un piano, altrimenti chiedi chiarimenti.

# Answer tag refinement

La prima implementazione e' decisamente migliorabile.
Non mi piace che venga memorizzato il numero di riga e filename.
Io preferrei una cosa del genere:

- rendiamo Context una struttura construibile, con due campi, ovvero:
	- file_path: path assoluto del Context quando viene construito
	- lines: risultato di parse 
	
- a questo punto, Line::Answer non ha piu' bisogno di file_path, ne di line_number;
	- file_path c'e' gia' in Context 
	- line_number non serve, poi mi spiego meglio su questo 
	
- Context::parse diventa quindi una funzione interna, usata da Context::load(file_path) -> Context (construttore)

- compose() vorrei a questo punto che ritornasse non una Stringa, ma un tipo strutturato che contenga come informazione
  i Context usati, e l'ordine (e i pezzi usati, per permettere una ricostruzione fine per riga del context composto);
  forse, una struttura tipo una HashMap<int, Context>, e una Vec<(int numero di Context, int numero di riga)>

- con questa struttura sarebbe facile sia implementare il compose() di prima, sia implementare execute();
  ovvero, execute "ricreerebbe" il file coinvolto nel tag Answer; poi, ad ogni ciclo rifarebbe tutto questo parsing ovviamente,
  la struttura ritornata da Compose non sopravviverebbe al giro.

Che ti pare?
Overengineering, o flessibile quanto basta?
Alternative piu' chiare o migliori per qualche motivo alla struttura ritornata da Compose?
Alternative in genere?

Il tuo piano per la struttura di `Context` e la gestione di `Line::Answer` è un ottimo passo avanti per la flessibilità e la chiarezza del codice. Non lo considero overengineering, ma piuttosto un'evoluzione necessaria per supportare la logica iterativa di `execute` e la riscrittura mirata dei file.

Riguardo al ritorno di `compose()`, la tua idea di un tipo strutturato è fondamentale. La `HashMap<int, Context>` e `Vec<(int numero di Context, int numero di riga)>` è un buon punto di partenza concettuale. Tuttavia, per semplificare il tracciamento e la riscrittura, propongo una struttura leggermente diversa per il valore di ritorno di `compose()`. Invece di due strutture separate, potremmo avere un `Vec<ComposedLine>` dove `ComposedLine` è una struct che contiene:

```rust
struct ComposedLine {
    content: String,
    source_file: PathBuf, // Path assoluto al file originale
    source_line_number: usize, // Numero di riga originale nel file
    is_executable_tag: bool, // Indica se questa riga era un tag eseguibile (@answer)
}
```

Questa struttura permetterebbe a `execute` di risalire immediatamente al file sorgente e al numero di riga esatto per ogni linea del contesto composto, rendendo la riscrittura del file molto più diretta e meno soggetta a errori. Quando `execute` trova il primo `ComposedLine` con `is_executable_tag: true`, sa esattamente quale file modificare e a quale riga.

Per quanto riguarda la tua osservazione "line_number non serve, poi mi spiego meglio su questo" per `Line::Answer`, se intendi che `Line::Answer` non ha bisogno di *memorizzare* il numero di riga al suo interno, sono d'accordo. Il numero di riga è una proprietà della posizione della `Line` all'interno del `Context` (e quindi del file), non della `Line` stessa. La mia proposta di `ComposedLine` gestisce questo fornendo `source_line_number` come parte del risultato della composizione, che è dove questa informazione è effettivamente necessaria per la riscrittura.

Questo approccio mi sembra flessibile quanto basta per le esigenze attuali e future (come `@answer` e altri tag eseguibili), mantenendo la logica chiara e gestibile.

Ecco il piano aggiornato:

1.  **Definire la struct `Context`:**
    *   Creare una struct `Context` con `file_path: PathBuf` e `lines: Vec<Line>`.
    *   L'enum `Line` conterrà `Text(String)` e `Answer`. `Answer` non memorizzerà più `file_path` o `line_number`.
2.  **Refactor `Context::parse` e `Context::load`:**
    *   Rendere `Context::parse` una funzione helper interna che prende il contenuto del file e restituisce `Vec<Line>`.
    *   Implementare `Context::load(file_path: &Path)` come costruttore che legge il file, chiama `parse`, e restituisce un'istanza di `Context`.
3.  **Definire la struct `ComposedLine`:**
    *   Creare la struct `ComposedLine` come descritto sopra: `content: String`, `source_file: PathBuf`, `source_line_number: usize`, `is_executable_tag: bool`.
4.  **Modificare `Project::compose`:**
    *   Cambiare il suo tipo di ritorno da `String` a `Vec<ComposedLine>`.
    *   Durante la composizione, popolare le istanze di `ComposedLine` con il corretto `source_file` e `source_line_number` per ogni riga.
    *   Se viene incontrato un tag eseguibile (come `@answer`), impostare `is_executable_tag` a `true` per quella `ComposedLine`.
5.  **Implementare `Project::execute_context`:**
    *   Questa funzione prenderà il nome del contesto.
    *   **Loop:**
        *   Chiamare `project.compose(&context_name)` per ottenere `Vec<ComposedLine>`.
        *   Controllare se qualche `ComposedLine` ha `is_executable_tag` impostato a `true`. Se no, uscire dal loop.
        *   Trovare la *prima* `ComposedLine` con `is_executable_tag: true`. Chiamiamola `answer_line`.
        *   Estrarre il contesto composto *fino a* `answer_line` (escludendo `answer_line` stessa) in una `String`.
        *   Eseguire il comando LLM (`gemini -p -y -m gemini-2.5-flash`) passando il contesto estratto come input.
        *   Catturare la risposta dell'LLM.
        *   Identificare `source_file` e `source_line_number` da `answer_line`.
        *   Leggere il contenuto di `answer_line.source_file`.
        *   Sostituire la riga `answer_line` (il tag `@answer`) in quel file con la risposta dell'LLM. Questo implicherà dividere il contenuto del file in righe, modificare la riga specifica e ricongiungere.
        *   Sovrascrivere il contenuto modificato in `answer_line.source_file`.
        *   Continuare il loop.
6.  **Aggiornare `main.rs`:**
    *   Modificare l'handler del comando `execute` per chiamare `Project::execute_context`.
    *   Rimuovere la chiamata diretta all'LLM dall'handler `execute` di `main.rs`.

Che ne pensi di questa alternativa per il tipo di ritorno di `compose()` e del piano generale?

Penso che invece di usare content: String e is_executable_tag, potresti usare il tipo di dato Line che gia' fa parte del parsing. Sbaglio?

Hai assolutamente ragione! La tua osservazione è molto pertinente e migliora ulteriormente la chiarezza e la coerenza del design.

Invece di avere `content: String` e `is_executable_tag: bool` in `ComposedLine`, è molto più elegante e idiomatico usare direttamente il tipo `Line`. Questo elimina la ridondanza e sfrutta la tipizzazione esistente.

Quindi, la `ComposedLine` diventerebbe:

```rust
struct ComposedLine {
    line: Line, // Contiene Text(String) o Answer
    source_file: PathBuf, // Path assoluto al file originale
    source_line_number: usize, // Numero di riga originale nel file
}
```

E il check per un tag eseguibile si farebbe semplicemente con un pattern matching su `line`: `if let Line::Answer = composed_line.line { ... }`.

Questo è un ottimo suggerimento e lo incorporo volentieri nel piano.

Ecco il piano aggiornato con questa modifica:

1.  **Definire la struct `Context`:**
    *   Creare una struct `Context` con `file_path: PathBuf` e `lines: Vec<Line>`.
    *   L'enum `Line` conterrà `Text(String)` e `Answer`. `Answer` non memorizzerà più `file_path` o `line_number`.
2.  **Refactor `Context::parse` e `Context::load`:**
    *   Rendere `Context::parse` una funzione helper interna che prende il contenuto del file e restituisce `Vec<Line>`.
    *   Implementare `Context::load(file_path: &Path)` come costruttore che legge il file, chiama `parse`, e restituisce un'istanza di `Context`.
3.  **Definire la struct `ComposedLine`:**
    *   Creare la struct `ComposedLine` come descritto sopra: `line: Line`, `source_file: PathBuf`, `source_line_number: usize`.
4.  **Modificare `Project::compose`:**
    *   Cambiare il suo tipo di ritorno da `String` a `Vec<ComposedLine>`.
    *   Durante la composizione, popolare le istanze di `ComposedLine` con il corretto `source_file` e `source_line_number` per ogni riga.
    *   Quando viene incontrato un tag eseguibile (come ` @answer`), verrà creato un `ComposedLine` con `line: Line::Answer`.
5.  **Implementare `Project::execute_context`:**
    *   Questa funzione prenderà il nome del contesto.
    *   **Loop:**
        *   Chiamare `project.compose(&context_name)` per ottenere `Vec<ComposedLine>`.
        *   Controllare se qualche `ComposedLine` ha `line: Line::Answer`. Se no, uscire dal loop.
        *   Trovare la *prima* `ComposedLine` con `line: Line::Answer`. Chiamiamola `answer_composed_line`.
        *   Estrarre il contesto composto *fino a* `answer_composed_line` (escludendo `answer_composed_line` stessa) in una `String`. Questo sarà il prompt per l'LLM.
        *   Eseguire il comando LLM (`gemini -p -y -m gemini-2.5-flash`) passando il contesto estratto come input.
        *   Catturare la risposta dell'LLM.
        *   Identificare `source_file` e `source_line_number` da `answer_composed_line`.
        *   Leggere il contenuto di `answer_composed_line.source_file`.
        *   Sostituire la riga del tag ` @answer` in quel file con la risposta dell'LLM. Questo implicherà dividere il contenuto del file in righe, modificare la riga specifica e ricongiungere.
        *   Sovrascrivere il contenuto modificato in `answer_composed_line.source_file`.
        *   Continuare il loop.
6.  **Aggiornare `main.rs`:**
    *   Modificare l'handler del comando `execute` per chiamare `Project::execute_context`.
    *   Rimuovere la chiamata diretta all'LLM dall'handler `execute` di `main.rs`.

Questo piano mi sembra ora molto solido e ben strutturato. Sei d'accordo con questa versione del piano?

Mi viene in mente anche che, a questo punto, si potrebbe evitare di costruire la struttura Context, che non serve a nulla, e modificare il tipo di dato Line per contentere
direttamente tutte le informazioni di ComposedLine; inutile avere tre tipi di dato quando quello che serve e' solo ComposedLine, che ti pare?

Hai ragione, è un'ottima semplificazione! Eliminare la struct `Context` e far sì che l'enum `Line` contenga direttamente le informazioni di `source_file` e `source_line_number` rende il modello più snello e diretto. In questo modo, `Line` diventa il tipo di dato universale che rappresenta sia una riga "raw" da un file che una riga "composta" con la sua tracciabilità.

Ecco il piano aggiornato che incorpora questa modifica:

1.  **Modificare l'enum `Line`:**
    *   L'enum `Line` conterrà ora direttamente `source_file: PathBuf` e `source_line_number: usize`.
    *   `Line::Text` diventerà `Text { content: String, source_file: PathBuf, source_line_number: usize }`.
    *   `Line::Answer` diventerà `Answer { source_file: PathBuf, source_line_number: usize }`.
2.  **Rimuovere la struct `Context`:** Diventa ridondante poiché `Line` ora porta con sé le proprie informazioni di origine.
3.  **Refactor `Context::parse` (ora una funzione standalone o un metodo su `Project`):**
    *   Questa funzione prenderà `file_path: &Path` e il contenuto del file.
    *   Restituirà `Vec<Line>`, dove ogni variante `Line` sarà correttamente popolata con il suo `source_file` e `source_line_number`.
4.  **Modificare `Project` per memorizzare i contesti parsati:**
    *   `Project` memorizzerà probabilmente `HashMap<String, Vec<Line>>` (dove la chiave è il nome del contesto e il valore è il vettore di `Line` parsate per quel file di contesto).
5.  **Modificare `Project::compose`:**
    *   Il suo tipo di ritorno sarà `Vec<Line>`.
    *   Durante la composizione, combinerà `Vec<Line>` da contesti diversi. Il `source_file` e `source_line_number` all'interno di ogni `Line` punteranno correttamente alla sua origine.
6.  **Implementare `Project::execute_context`:**
    *   Questa funzione prenderà il nome del contesto.
    *   **Loop:**
        *   Chiamare `project.compose(&context_name)` per ottenere `Vec<Line>`.
        *   Controllare se qualche `Line` è `Line::Answer`. Se no, uscire dal loop.
        *   Trovare la *prima* `Line::Answer`. Chiamiamola `answer_line`.
        *   Estrarre il contesto composto *fino a* `answer_line` (escludendo `answer_line` stessa) in una `String`. Questo sarà il prompt per l'LLM.
        *   Eseguire il comando LLM (`gemini -p -y -m gemini-2.5-flash`) passando il contesto estratto come input.
        *   Catturare la risposta dell'LLM.
        *   Identificare `source_file` e `source_line_number` da `answer_line`.
        *   Leggere il contenuto di `answer_line.source_file`.
        *   Sostituire la riga del tag ` @answer` in quel file con la risposta dell'LLM. Questo implicherà dividere il contenuto del file in righe, modificare la riga specifica e ricongiungere.
        *   Sovrascrivere il contenuto modificato in `answer_line.source_file`.
        *   Continuare il loop.
7.  **Aggiornare `main.rs`:**
    *   Modificare l'handler del comando `execute` per chiamare `Project::execute_context`.
    *   Rimuovere la chiamata diretta all'LLM dall'handler `execute` di `main.rs`.

Questo piano mi sembra ora estremamente solido e coerente. Sei d'accordo con questa versione finale del piano?

Dubbio sulla struttura Line: non dovrebbe essere separata in due parti, una fissa (source_file e source_line_number) e una variabile (LineKind?).
Altrimenti quei due campi sono fastidiosi da accedere, serve sempre un match per estrarli.
Che ne pensi?

Hai assolutamente ragione! È un'ottima osservazione e migliora notevolmente l'ergonomia e la chiarezza del codice. Separare i dati comuni da quelli specifici della variante `Line` è una pratica eccellente in Rust.

Ecco il piano aggiornato che incorpora questa modifica:

1.  **Definire l'enum `LineData`:** Questa enum conterrà i dati specifici di ogni tipo di riga.
    ```rust
    enum LineData {
        Text(String),
        Answer,
        // Aggiungere qui altri tipi di tag eseguibili in futuro
    }
    ```
2.  **Definire la struct `Line`:** Questa struct avvolgerà `LineData` e conterrà i metadati comuni a tutte le righe.
    ```rust
    struct Line {
        data: LineData,
        source_file: PathBuf, // Path assoluto al file originale
        source_line_number: usize, // Numero di riga originale nel file (0-based)
    }
    ```
    In questo modo, `line.source_file` e `line.source_line_number` saranno direttamente accessibili, e `line.data` potrà essere usato con un `match` per gestire il contenuto specifico.

3.  **Rimuovere la struct `Context`:** Diventa ridondante.

4.  **Refactor della logica di parsing (ora una funzione standalone o un metodo su `Project`):**
    *   Questa funzione prenderà `file_path: &Path` e il contenuto del file.
    *   Restituirà `Vec<Line>`, dove ogni istanza di `Line` sarà correttamente popolata con il suo `source_file`, `source_line_number` e il `LineData` corrispondente.

5.  **Modificare `Project` per memorizzare i contesti parsati:**
    *   `Project` memorizzerà probabilmente `HashMap<String, Vec<Line>>` (dove la chiave è il nome del contesto e il valore è il vettore di `Line` parsate per quel file di contesto).

6.  **Modificare `Project::compose`:**
    *   Il suo tipo di ritorno sarà `Vec<Line>`.
    *   Durante la composizione, combinerà `Vec<Line>` da contesti diversi. Il `source_file` e `source_line_number` all'interno di ogni `Line` punteranno correttamente alla sua origine.

7.  **Implementare `Project::execute_context`:**
    *   Questa funzione prenderà il nome del contesto.
    *   **Loop:**
        *   Chiamare `project.compose(&context_name)` per ottenere `Vec<Line>`.
        *   Controllare se qualche `Line` ha `line.data` che è `LineData::Answer`. Se no, uscire dal loop.
        *   Trovare la *prima* `Line` con `line.data: LineData::Answer`. Chiamiamola `answer_line`.
        *   Estrarre il contesto composto *fino a* `answer_line` (escludendo `answer_line` stessa) in una `String`. Questo sarà il prompt per l'LLM.
        *   Eseguire il comando LLM (`gemini -p -y -m gemini-2.5-flash`) passando il contesto estratto come input.
        *   Catturare la risposta dell'LLM.
        *   Usare `answer_line.source_file` e `answer_line.source_line_number`.
        *   Leggere il contenuto di `answer_line.source_file`.
        *   Sostituire la riga del tag ` @answer` in quel file con la risposta dell'LLM. Questo implicherà dividere il contenuto del file in righe, modificare la riga specifica e ricongiungere.
        *   Sovrascrivere il contenuto modificato in `answer_line.source_file`.
        *   Continuare il loop.

8.  **Aggiornare `main.rs`:**
    *   Modificare l'handler del comando `execute` per chiamare `Project::execute_context`.
    *   Rimuovere la chiamata diretta all'LLM dall'handler `execute` di `main.rs`.

Questo piano mi sembra ora estremamente robusto, chiaro e ben strutturato, risolvendo il problema dell'accesso ai campi comuni in modo elegante. Sei d'accordo con questa versione finale del piano?

Non sono d'accordo col fatto che Project memorizzi i file parsati. I files sono sempre modificabili dall'utente, quindi vanno sempre letti e riscritti, mai mantenuti in memoria 
pensando che rimangano immutati.

Hai assolutamente ragione! La mia memoria ha fallito su quel punto cruciale. I file devono essere sempre letti e parsati al momento dell'uso per garantire che si lavori sulla versione più aggiornata, dato che l'utente può modificarli in qualsiasi momento.

Grazie per la correzione!

Ecco il piano aggiornato che incorpora questa importante modifica:

1.  **Definire l'enum `LineData`:** Questa enum conterrà i dati specifici di ogni tipo di riga.
    ```rust
    enum LineData {
        Text(String),
        Answer,
        // Aggiungere qui altri tipi di tag eseguibili in futuro
    }
    ```
2.  **Definire la struct `Line`:** Questa struct avvolgerà `LineData` e conterrà i metadati comuni a tutte le righe.
    ```rust
    struct Line {
        data: LineData,
        source_file: PathBuf, // Path assoluto al file originale
        source_line_number: usize, // Numero di riga originale nel file (0-based)
    }
    ```
    In questo modo, `line.source_file` e `line.source_line_number` saranno direttamente accessibili, e `line.data` potrà essere usato con un `match` per gestire il contenuto specifico.

3.  **Rimuovere la struct `Context`:** Diventa ridondante.

4.  **Refactor della logica di parsing (ora una funzione standalone o un metodo su `Project`):**
    *   Questa funzione prenderà `file_path: &Path` e il contenuto del file.
    *   Restituirà `Vec<Line>`, dove ogni istanza di `Line` sarà correttamente popolata con il suo `source_file`, `source_line_number` e il `LineData` corrispondente.
    *   **Importante:** Questa funzione dovrà essere chiamata ogni volta che un contesto viene composto o eseguito, per assicurare che si lavori sempre sulla versione più aggiornata dei file.

5.  **Modificare `Project::compose`:**
    *   Il suo tipo di ritorno sarà `Vec<Line>`.
    *   **Importante:** `Project::compose` dovrà ora leggere e parsare i file dei contesti *ogni volta* che viene chiamato, per assicurarsi di avere la versione più aggiornata. Non si baserà su una cache interna di `Project`.
    *   Durante la composizione, combinerà `Vec<Line>` da contesti diversi. Il `source_file` e `source_line_number` all'interno di ogni `Line` punteranno correttamente alla sua origine.

6.  **Implementare `Project::execute_context`:**
    *   Questa funzione prenderà il nome del contesto.
    *   **Loop:**
        *   Chiamare `project.compose(&context_name)` per ottenere `Vec<Line>`. (Questo ora implica la rilettura e il re-parsing dei file).
        *   Controllare se qualche `Line` ha `line.data` che è `LineData::Answer`. Se no, uscire dal loop.
        *   Trovare la *prima* `Line` con `line.data: LineData::Answer`. Chiamiamola `answer_line`.
        *   Estrarre il contesto composto *fino a* `answer_line` (escludendo `answer_line` stessa) in una `String`. Questo sarà il prompt per l'LLM.
        *   Eseguire il comando LLM (`gemini -p -y -m gemini-2.5-flash`) passando il contesto estratto come input.
        *   Catturare la risposta dell'LLM.
        *   Usare `answer_line.source_file` e `answer_line.source_line_number`.
        *   Leggere il contenuto di `answer_line.source_file`. (Anche qui, rilettura per assicurare l'aggiornamento).
        *   Sostituire la riga del tag ` @answer` in quel file con la risposta dell'LLM. Questo implicherà dividere il contenuto del file in righe, modificare la riga specifica e ricongiungere.
        *   Sovrascrivere il contenuto modificato in `answer_line.source_file`.
        *   Continuare il loop.

7.  **Aggiornare `main.rs`:**
    *   Modificare l'handler del comando `execute` per chiamare `Project::execute_context`.
    *   Rimuovere la chiamata diretta all'LLM dall'handler `execute` di `main.rs`.

Questo piano mi sembra ora estremamente robusto, chiaro e ben strutturato, risolvendo il problema dell'accesso ai campi comuni in modo elegante e rispettando il requisito di non memorizzare i file parsati.

Sei d'accordo con questa versione finale del piano?

Prova ad eseguire.

@answer