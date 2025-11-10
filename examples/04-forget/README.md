This project shows @answer used with choose, that forces llm to choose between a definite set of choices.
It also shows the @forget tag, that can be used to erase all collected context in current context file, and
re-start a new conversation from scratch in the same file.

.ctx/contexts/main.md							- this is the context as it is before being answered
.ctx/contexts/main_executed.md					- this is the same context after an example execution 
.ctx/contexts/agent/secretary.md				- this context gives llm personality o a loyal secretary 
.ctx/contexts/input/email.md					- this context contains an imaginary email received
.ctx/contexts/instructions/summarize.md			- this context contains instructions to summarize a context 
.ctx/contexts/instructions/names.md				- this context contains instructions to extract names from a context
.ctx/contexts/output/summarize.md				- this context contains the output of the summarize instructions applied to the email 
.ctx/contexts/output/names.md					- this context contains the output of the names instructions applied to the email 

 



