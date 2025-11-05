@include rules
@include agent/gemini_25_flash_yolo

Tell me 3 violet things.

<!-- answer-80089f74-20c9-4314-8466-a462e2c4b53d:begin { provider: "gemini -y -m gemini-2.5-flash" }  -->
Here are 3 violet things:

1.  **Amethyst:** A popular purple-to-violet variety of quartz.
2.  **Violet flower:** A small, typically purple or blue-purple flowering plant.
3.  **Grape:** Many varieties of grapes have a deep violet color.
<!-- answer-80089f74-20c9-4314-8466-a462e2c4b53d:end {}  -->

<!-- answer-657f5d63-4962-4335-8fa2-ae35a1516f5f:begin {
	input: test/orange,
	provider: "gemini -y -m gemini-2.5-flash"
}  -->
1. Orange (fruit)
2. Carrot
3. Pumpkin
4. Traffic cone
5. Monarch butterfly
<!-- answer-657f5d63-4962-4335-8fa2-ae35a1516f5f:end {}  -->

Tell me the difference between red and blue.

<!-- answer-370ce26b-59a9-41aa-8681-e55297033e70:begin {
	output: out/doggy,
	prefix: agent/doggy,
	provider: "gemini -y -m gemini-2.5-flash"
}  -->
<!-- answer-370ce26b-59a9-41aa-8681-e55297033e70:end {}  -->

Tell me something nice.

<!-- answer-23751734-31cb-453d-b485-3076e0a79a7b:begin {
	system: agent/gemini_25_flash_yolo,
	provider: "gemini -y -m gemini-2.5-flash"
}  -->
You are doing great! Keep up the excellent work.
<!-- answer-23751734-31cb-453d-b485-3076e0a79a7b:end {}  -->

Tell me the difference between yellow and green

<!-- answer-94ee142c-84e6-47c4-b1d6-6a672eff3e10:begin {
	provider: "gemini -y -m gemini-2.5-flash",
	output: out/kitty,
	prefix: agent/kitty
}  -->
<!-- answer-94ee142c-84e6-47c4-b1d6-6a672eff3e10:end {}  -->



