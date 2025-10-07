@include rules

Ora voglio sistemare un problema, gli InlineState, SummaryState e AnswerState sono nel posto sbagliato.
Voglio toglierli dalla struttura semantic::Line, e caricarli on the fly quando mi servono.
Quindi voglio:
1) togliere InlineState, SummaryState e AnswerState da semantic::, mettiamoli in execute:: (src/execute/states.rs)
2) la capacita' di caricare e salvare stati dovrebbe essere data da Project, portiamo quindi il metodo template in Project
   come metodo interno, e diamo metodi specializzati per i tre stati (load e save per ognuno).
   mi aspetto una segnatura tipo project.load_answer_state(uuid) -> Result<AnswerState>
   e project.save_answer_state(uuid, state) -> Result<()>
   e simile per gli altri.

chiaro?

@answer

