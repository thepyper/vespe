@comment {
    _1: "Run this with 'vespe context run main'.",
}

@set {
	with_agent_names,
	with_invitation,
}

@include agent/gemini_25_flash_yolo.md

Tell me something about blue.

<!-- answer-0f242727-c103-48c2-ae3a-efeaeeab0b62:begin +completed+ {
	prefix: agent/funny_clown.md,
	with_agent_names,
	with_invitation,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
Why, blue is the color of the sky, the ocean, and sometimes, when I've had too much cotton candy, even my tongue! It's a color so cool, it makes ice cubes jealous. And if you ever feel blue, just remember, a clown's red nose is always there to cheer you up! Honk honk!
<!-- answer-0f242727-c103-48c2-ae3a-efeaeeab0b62:end  {}  -->

Tell me something about red.

<!-- answer-a349a8f1-c885-4c5d-b02a-ed69ed63c098:begin +completed+ {
	with_agent_names,
	prefix: agent/creepy_crow.md,
	provider: 'gemini -y -m gemini-2.5-flash',
	with_invitation
}  -->
Craak! Red, you say? Ah, yes, the color of fresh, warm... sustenance. The hue of a hunter's moon, casting long, twisted shadows. It's the flush on a cheek, just before the chill sets in. Craak! A most *interesting* color, indeed.
<!-- answer-a349a8f1-c885-4c5d-b02a-ed69ed63c098:end  {}  -->


