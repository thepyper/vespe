In this project @inline is used to instantiate a template (about_color.md) filled with different parameters:

- first two instances get a direct parameter;
- third instance takes parameter from the second positional command-line argument; 
- fourth instance takes parameter from the whole positional parameters string;
- fifth instance takes parameter from standard input; 
- sixth instance takes parameter from a command-line definition flag

.vespe/contexts/main.md							    - this is the context as it is before being answered

.vespe/contexts/main_executed.md					- this is the same context after an example execution 

.vespe/contexts/template/about_color.md			    - this context is instantiated with @inline in main context 
 



