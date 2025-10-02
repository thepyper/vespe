@include ctx

# inline_refactor

La feature @inline mi soddisfa quasi.

E' venuta abbastanza bene, ma ha un problema:
- @inline correttamente include inline lo snippet
- tuttavia, nel file markdown viene sostituita la riga @inline dalla risposta del llm solamente,
  e non anche dello snippet;

praticamente quello che voglio e' che lo snippet venga di fatto incluso nel file richiamante,
e che quel file venga effettivamente modificato.

Come modifica mi viene in mente un cambiamento della logica di execute_context (in project.rs);
Ora la funzione riscrive il file solamente se trova un tag @answer.

La funzione potrebbe invece ricrivere SEMPRE il file, in questo modo:
- se trova linee Text, le copia cosi come sono;
- se trova una linea Inline, processa il file incluso ugualmente con execute, partendo dal contesto presente al momento dell'inlining;
- alla prima linea Answer che trova (anche se trovata durante l'inlining) lancia la chiamata llm per sostituirla con la risposta;
- ogni successiva linea Answer la copia cosi come e';

Praticamente "rigenera" il file ogni volta, facendo delle operazioni, ovvero:
- processando tutti gli @inline
- processando la prima @answer (anche trovata dopo inline)

Oppure, 2-pass:
- 1 pass processa gli inline e li sostituisce con i contenuti (operando sempre a livello di Vec<Line>)
- 2 pass processa copia tutto fino alla prima answer, poi copia tutto comprese le altre answer

La scelta piu' pulita? future-proof?


@answer

