@include rules

Prima di iniziare con la esecuzione vera e propria voglio una modifica a decorator.rs e injector.rs.
Le due funzioni le voglio entrambe spezzate in due:
- una funzione che lavori su un ingresso tipo mut Vec<Line> ed esegua la modifica in-memory; deve ritornare Result<bool> con bool = true se qualche modifica e' avvenuta;
- una facade che esegua la funzione sul file come avviene attualmente, usando la funzione separata;

chiaaro?

Capito. Devo refactorizzare due funzioni, una in `decorator.rs` e una in `injector.rs`, suddividendo ciascuna in una funzione in-memory (`mut Vec<Line> -> Result<bool>`) e una facade che opera su file.

Inizierò leggendo `src/decorator.rs` per identificare la funzione da modificare.La funzione `decorate_context` in `src/decorator.rs` necessita di refactoring. Attualmente legge, modifica e riscrive un file.

Il piano per `src/decorator.rs` è:

1.  **Creare `decorate_context_in_memory`:**
    *   Accetterà `mut Vec<Line>`.
    *   Conterrà la logica di modifica ("First Pass" e "Second Pass").
    *   Restituirà `Result<bool>` (true se modificato).

2.  **Modificare `decorate_context` (facade):**
    *   Manterrà la gestione I/O del file.
    *   Leggerà e parserà il file in `Vec<Line>`.
    *   Chiamerà `decorate_context_in_memory`.
    *   Se modificato, formatterà e scriverà il contenuto.
    *   Restituirà `Result<()>`.

Inizio con l'implementazione di `decorate_context_in_memory`.Ho refactorizzato `src/decorator.rs`. Ora devo fare lo stesso per `src/injector.rs`. Leggerò il contenuto di `src/injector.rs` per identificare la funzione da refactorizzare.La funzione `inject_content` in `src/injector.rs` necessita di refactoring. Attualmente legge, modifica e riscrive un file.

Il piano per `src/injector.rs` è:

1.  **Creare `inject_content_in_memory`:**
    *   Accetterà `mut Vec<Line>`, `anchor_kind`, `anchor_uid` e `new_content`.
    *   Conterrà la logica per trovare gli anchor, rimuovere il contenuto esistente e inserire il nuovo.
    *   Restituirà `Result<bool>` (true se modificato).

2.  **Modificare `inject_content` (facade):**
    *   Manterrà la gestione I/O del file.
    *   Leggerà e parserà il file in `Vec<Line>`.
    *   Chiamerà `inject_content_in_memory` con i parametri appropriati.
    *   Se modificato, formatterà e scriverà il contenuto.
    *   Restituirà `Result<()>`.

Procedo con il refactoring.Ho refactorizzato `src/decorator.rs` e `src/injector.rs`. Ora eseguo `cargo check` per verificare la compilazione.Ci sono errori di compilazione in `src/injector.rs` relativi al tipo di ritorno `Box<dyn std::error::Error>`. L'errore `expected one of `,` or `>`, found `.` è fuorviante; il problema è l'uso di `std::error.Error` invece di `std::error::Error`. Correggo il tipo di ritorno della funzione `inject_content_in_memory` in `src/injector.rs` sostituendo `.` con `::`.Ho corretto l'errore di battitura in `src/injector.rs`. Ora eseguo `cargo check` per verificare la correzione.Ottimo! Il progetto compila. I warning sono trascurabili. Ho refactorizzato `decorator.rs` e `injector.rs`. Ora eseguo il commit.`git status` e `git diff HEAD` mostrano le modifiche a `src/decorator.rs` e `src/injector.rs`. Ora le stagiono e le commetto con un messaggio descrittivo.I file sono stati preparati. Ora li commetto.Mi scuso, ho sbagliato il comando di commit usando una stringa multilinea non supportata. Avrei dovuto usare una singola stringa con newline escape o flag `-m` separati. Riprovo con il formato corretto.Mi scuso ancora. Ho problemi con i messaggi di commit multilinea. Il tool `run_shell_command` non interpreta `\n` come newline. Per ora, farò un commit con un messaggio conciso su una riga. Se l'utente desidera un messaggio più dettagliato, fornirò istruzioni o un approccio diverso.Il commit è riuscito. Ho refactorizzato `decorator.rs` e `injector.rs` suddividendo le funzioni in versioni in-memory e facade, e il progetto compila senza errori. Ho anche commesso le modifiche su git. Credo di aver soddisfatto la richiesta dell'utente. Attendo ulteriori istruzioni.

Ora, iniziamo ad implementare l'esecuzione.
Prima di tutto creiamo un modulo in src/execute/mod.rs

Dentro ci va una funzione pubblica: fn execute(project, context_name);

Questa funzione ha un internal loop del genere:

execute(project, context_name) {

    // carica context context_name come Vec<Line>

    chiama decorate_recursive_file(context_name)
    
    // altre cose da fare dopo


}

decorate_recursive_file(context_name, decorated_set)
{    
    esegui decorate(context_name)
    
    carica context_name come Vec<Line>

    ciclo su linee, follow di @include per eseguire decorazione ricorsiva
    usa decorated_set contro loop circolari

}

Ok, fatto

Ora che hai fatto, implementa inject_recursive_inline(context_name)

inject_recursive_inline(context_name, inlined_set)
{
    // carica context_name

    cicla sulle linee, trova quelle con tag inline, con ancora begin

    per ogni tag inline, carica lo snippet riferito, e copia le linee dello snippet dentro le ancore usando injector.rs::inject_content

    // cicla poi sulle linee per eseguire ricorsivamente su @include

}

@answer





