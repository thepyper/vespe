@include rules
@include agent/gemini_25_flash_yolo

Tell me 3 violet things.

<!-- answer-85c12df7-2f6d-447b-b644-081e00062b1c:begin { provider: "gemini -y -m gemini-2.5-flash" }  -->
Here are 3 violet things:

1.  Amethyst (a gemstone)
2.  Lavender (a flower)
3.  Violet (a color, also a flower)
<!-- answer-85c12df7-2f6d-447b-b644-081e00062b1c:end {}  -->

<!-- answer-841805d4-88e6-457d-925a-8fcee4eb0414:begin {
	input: test/orange,
	provider: "gemini -y -m gemini-2.5-flash"
}  -->
Here are 5 orange objects:

1.  An orange (fruit)
2.  A traffic cone
3.  A monarch butterfly
4.  A pumpkin
5.  A carrot
<!-- answer-841805d4-88e6-457d-925a-8fcee4eb0414:end {}  -->

Tell me the difference between red and blue.

<!-- answer-da01760d-26dd-4d58-868e-dc785d8b4516:begin {
	output: out/doggy,
	provider: "gemini -y -m gemini-2.5-flash",
	prefix: agent/doggy
}  -->
<!-- answer-da01760d-26dd-4d58-868e-dc785d8b4516:end {}  -->

Tell me something nice.

<!-- answer-804c0666-eea9-4281-a29b-69003a6a65ae:begin {
	provider: "gemini -y -m gemini-2.5-flash",
	system: agent/gemini_25_flash_yolo
}  -->
You are doing great! Keep up the fantastic work.
<!-- answer-804c0666-eea9-4281-a29b-69003a6a65ae:end {}  -->

Tell me the difference between yellow and green

<!-- answer-ceccca51-47f1-4eff-8dbb-2840dd66cc7c:begin {
	output: out/kitty,
	prefix: agent/kitty,
	provider: "gemini -y -m gemini-2.5-flash"
}  -->
<!-- answer-ceccca51-47f1-4eff-8dbb-2840dd66cc7c:end {}  -->



