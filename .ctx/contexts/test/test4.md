@include rules
@include agent/gemini_25_flash_yolo

Which one is most able in bending?

1. Bender
2. Fry

<!-- answer-86ddcc64-51ae-4d53-ad04-8b33a97d34d1:begin {
	provider: "gemini -y -m gemini-2.5-flash",
	choose: {
		fry: "Fry the Wimpy.",
		bender: "Bender the Offender!!!!"
	}
}  -->
<!-- answer-86ddcc64-51ae-4d53-ad04-8b33a97d34d1:end {}  -->

Here is a list of high things:
1. Everest
2. My dog
3. Tour Eifell

Which is the tallest?

@answer {
	choose: [1,2,3]
}

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



