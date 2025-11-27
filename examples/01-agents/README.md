This project @include is used to include a setting, and prefix context (system prompt) is used 
in @answer tags to reply with different personalities to different parts of the conversation.
@set tag is used as well to enable invitation and agent names, which help models keep
track of different personas in the same conversation.

.vespe/contexts/main.md							    - this is the context as it is before being answered

.vespe/contexts/main_executed.md					- this is the same context after an example execution 

.vespe/contexts/agent/gemini_25_flash_yolo.md		- this context uses the @set tag to set the llm provider to gemini-2.5-flash

.vespe/contexts/agent/funny_clown.md				- this context is a simple system prompt to give a personality to llm

.vespe/contexts/agent/creepy_crow.md				- this context is a simple system prompt to give a personality to llm
 



