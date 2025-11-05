@include rules
@include agent/gemini_25_flash_yolo

Tell me 3 violet things.

@answer

@answer { input: test/orange }  

Tell me the difference between red and blue.

@answer {
	prefix: agent/doggy,
	output: out/doggy
}  

Tell me something nice.

@answer { system: agent/gemini_25_flash_yolo }  

Tell me the difference between yellow and green

@answer {
	prefix: agent/kitty,
	output: out/kitty
} 



