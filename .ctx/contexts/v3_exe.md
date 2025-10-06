@include rules

Prima di iniziare con la esecuzione vera e propria voglio una modifica a decorator.rs e injector.rs.
Le due funzioni le voglio entrambe spezzate in due:
- una funzione che lavori su un ingresso tipo mut Vec<Line> ed esegua la modifica in-memory; deve ritornare Result<bool> con bool = true se qualche modifica e' avvenuta;
- una facade che esegua la funzione sul file come avviene attualmente, usando la funzione separata;

chiaaro?

@answer

    

Ora, iniziamo ad implementare l'esecuzione.
Prima di tutto creiamo un modulo in src/execute/mod.rs

Intanto iniziamo con una parte, src/execute/inliner.rs
