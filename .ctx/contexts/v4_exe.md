
@include rules.md

Sto riscrivendo una importante funzione.
L'originale (circa funzionante ma non mi piace) e' in src/execute/mod.rs, e' execute();
La nuova e' nello stesso file, execute2();

Mettiamo un po' di ordine in _execute2, ci sono degli scope opportunamente separati per essere
fattorizzati in funzioni separate.

In ordine, separali in funzioni:
- decorate_with_new_anchors()
- check_for_orphan_anchors()
- apply_inline()
- apply_answer_summary()

chiaro?

@answer

