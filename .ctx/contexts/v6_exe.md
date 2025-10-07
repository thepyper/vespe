@include rules

Prossimo improvement e' spostare ContextManager direttamente in Project.
Ho deciso che invece di essere una struttura locale usata in alcune funzioni,
la capacita' di ContextManager di tenere una cache locale in-memory dei context
vada bene per Project. E sia globale.
Sposta quindi le capacita' di ContextManager in Project.
Una funzione aggiuntiva che deve essere disponibile e' un flush_modified() che 
scriva su file tutti i context marcati come modificati e cancelli poi
la memoria dei modificati.

!!! no se no diventa god object, non buono.



! parsing aggiungere ai Tag Inline{ path }, finire parsing argomenti in parser!

! parsing aggiungere State a ancore? gestire da la ?

stratificazione del parser? arricchimento della struttura parsed?

