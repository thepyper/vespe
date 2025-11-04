

1. @answer con i seguenti parametri:

prefix= (system di adesso, cambia nome!)
postfix= (aggiunto dopo context, istruzioni, sostituisce derive!)
choose= [lista di opzioni selezionabili] -> se c'e' fa ricerca di una opzione unica
choose= { come choose + mappatura ad una stringa fissa } -> choose puo' essere array o oggetto


2. sugar (implementato chiamando AnswerPolicy da OtherPolicy)

- @derive ... come si desugara? e' proprio uguale ad answer!   @answer postfix=   ===  @derive instructions=     
- @choose ... come? e' proprio uguale? o facciamo numerato? tanto per avere uno standard.
- @decide -> yes/no

- oppure sono tutti degli inline? mi sa di si.... @inline answer/choose;    


