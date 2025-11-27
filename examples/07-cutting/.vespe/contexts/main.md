%% Run this with 'vespe context run main'

===== instance 1
@inline { begin_line: 7, end_line: 8 } other.md
=====

===== instance 2
@inline { begin_marker: "+++1", end_marker: "+++2" } other.md
=====

===== instance 3
@inline { begin_marker: "+++3", begin_line: 1, end_marker: "+++4", end_line: -2 } other.md
=====

===== instance 4
@inline other.md
=====

