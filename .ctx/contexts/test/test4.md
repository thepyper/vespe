@include rules
@include agent/gemini_25_flash_yolo

<!-- inline-170467b1-7286-4eef-a455-c1f9b1b6bd11:begin { provider: "gemini -y -m gemini-2.5-flash" } test/blue -->
Tell me 4 blue things.
<!-- answer-1a0cb568-2b94-4c5a-87e4-8e3b55f32ad7:begin { provider: "gemini -y -m gemini-2.5-flash" }  -->
Here are 4 blue things:

1.  Sky
2.  Ocean
3.  Blueberries
4.  Denim jeans
<!-- answer-1a0cb568-2b94-4c5a-87e4-8e3b55f32ad7:end {}  -->

<!-- inline-170467b1-7286-4eef-a455-c1f9b1b6bd11:end {}  -->

Which one is most able in bending?

1. Bender
2. Fry

<!-- answer-28984e1f-62dd-4f98-87df-8f6d29b17ba9:begin {
	choose: {
		fry: "Fry the Wimpy.",
		bender: "Bender the Offender!!!!"
	},
	provider: "gemini -y -m gemini-2.5-flash"
}  -->
Bender the Offender!!!!
<!-- answer-28984e1f-62dd-4f98-87df-8f6d29b17ba9:end {}  -->

Here is a list of high things:
1. Everest
2. My dog
3. Tour Eifell

Which is the tallest?

<!-- answer-d6192b3e-4b97-481c-bf1d-1b7f6c493b7f:begin {
	choose: [
		1,
		2,
		3
	],
	provider: "gemini -y -m gemini-2.5-flash"
}  -->
§1§
<!-- answer-d6192b3e-4b97-481c-bf1d-1b7f6c493b7f:end {}  -->

Tell me 3 violet things.

<!-- answer-10733938-1d95-4667-826d-f5187676d6a5:begin { provider: "gemini -y -m gemini-2.5-flash" }  -->
Here are 3 violet things:

1.  Amethyst
2.  Violets (the flower)
3.  Grape
<!-- answer-10733938-1d95-4667-826d-f5187676d6a5:end {}  -->

<!-- answer-ea0fcc24-17bc-4910-91ba-6ebf61c0dd6c:begin {
	input: test/orange,
	provider: "gemini -y -m gemini-2.5-flash"
}  -->
Here are 5 orange objects:

1.  Orange (fruit)
2.  Carrot
3.  Pumpkin
4.  Traffic cone
5.  Monarch butterfly
<!-- answer-ea0fcc24-17bc-4910-91ba-6ebf61c0dd6c:end {}  -->

Tell me the difference between red and blue.

<!-- answer-cc930757-3e65-4e56-8a25-8213a9bcd5bd:begin {
	prefix: agent/doggy,
	output: out/doggy,
	provider: "gemini -y -m gemini-2.5-flash"
}  -->
<!-- answer-cc930757-3e65-4e56-8a25-8213a9bcd5bd:end {}  -->

Tell me something nice.

<!-- answer-96ef4bb5-75bd-4f2d-974e-7fd50fce3925:begin {
	provider: "gemini -y -m gemini-2.5-flash",
	system: agent/gemini_25_flash_yolo
}  -->
You are doing great! Keep up the fantastic work.
<!-- answer-96ef4bb5-75bd-4f2d-974e-7fd50fce3925:end {}  -->

Tell me the difference between yellow and green

<!-- answer-e9cc2f6a-d0c7-434e-897a-f3db3aed6296:begin {
	provider: "gemini -y -m gemini-2.5-flash",
	prefix: agent/kitty,
	output: out/kitty
}  -->
<!-- answer-e9cc2f6a-d0c7-434e-897a-f3db3aed6296:end {}  -->



