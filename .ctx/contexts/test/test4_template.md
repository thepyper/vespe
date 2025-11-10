@include rules
@include agent/gemini_25_flash_yolo

@include { data: { color: '{{$args}}' } } test/color2 
@answer

@inline { data: { color: 'lime', count: 2 } } test/color 

@comment { _:"Here is a comment that LLM will not read" }

@comment {
	_1: "Here is a multiline comment that LLM will not read",
	_2: "Hehehehe!!                                        ",
}

@inline test/blue

Tell me the color of pigs.

@answer { dynamic }

Which one is most able in bending?

1. Bender
2. Fry

@answer {
	choose: {
		bender: "Bender the Offender!!!!",
		fry: "Fry the Wimpy.",
	}
}

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



