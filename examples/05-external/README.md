This project shows using contexts from directories other than the .vespe directory, to organize prompt libraries. 
It also shows output directory redirection, to allow output outside the .vespe directory.

.vespe/contexts/main.md							    - this is the context as it is before being answered

.vespe/contexts/main_executed.md					- this is the same context after an example execution 

library                                             - this directory contains some contexts that are used in the project but being outside .vespe directory
library/agent/epoque.md         					- a very simple persona impersonating a guy of some other epoque

indir                                               - this directory contains some contexts that are used in the project but being outside .vespe directory
indir/epilogue.txt                                  - a file to inline in context

outdir                                              - this directory is used as output directory outside the .vespe directory
outdir/1200.txt                                     - output from first agent call
outdir/1980.txt                                     - output from second agent call
