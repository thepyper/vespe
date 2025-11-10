In this project @inline is used to instantiate a template (about_color.md) filled with different parameters.
First two instances get a direct parameter, third instance takes parameter from the first positional 
command-line argument; fourth instance takes parameter from the whole positional parameters string;
last instance takes parameter from standard input.

.ctx/contexts/main.md							- this is the context as it is before being answered

.ctx/contexts/main_executed.md					- this is the same context after an example execution 

.ctx/contexts/template/about_color.md			- this context is instantiated with @inline in main context 
 



