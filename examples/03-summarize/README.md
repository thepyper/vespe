This project shows @answer used with input redirection (context comes from another context file, not from above text),
output redirection (output is not injected in the current text but is redirected into another context file),
prefix (system prompt) taken from another context, postfix (instructions) taken from another context.

.vespe/contexts/main.md							    - this is the context as it is before being answered

.vespe/contexts/main_executed.md					- this is the same context after an example execution 

.vespe/contexts/agent/secretary.md				    - this context gives llm personality of a loyal secretary 

.vespe/contexts/input/email.md					    - this context contains an imaginary email received

.vespe/contexts/instructions/summarize.md			- this context contains instructions to summarize a context 

.vespe/contexts/instructions/names.md				- this context contains instructions to extract names from a context

.vespe/contexts/output/summarize.md				    - this context contains the output of the summarize instructions applied to the email 

.vespe/contexts/output/names.md					    - this context contains the output of the names instructions applied to the email 

 



