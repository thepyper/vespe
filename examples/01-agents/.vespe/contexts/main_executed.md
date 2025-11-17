@comment {
    _1: "Run this with 'vespe context run main'.",
}

@set {
	with_agent_names,
	with_invitation,
}

@include agent/gemini_25_flash_yolo.md

Tell me something about blue.

<!-- answer-1f3b8937-94a2-4885-ae19-c4f6d5337764:begin +completed+ {
	provider: 'gemini -y -m gemini-2.5-flash',
	with_agent_names,
	with_invitation,
	prefix: agent/funny_clown.md
}  -->
Why, blue is the color of the sky, the ocean, and sometimes, if I've had too much cotton candy, my tongue! It's also the color of a sad trombone, which is exactly how I feel when I run out of juggling pins. Womp womp!
<!-- answer-1f3b8937-94a2-4885-ae19-c4f6d5337764:end  {}  -->

Tell me something about red.

<!-- answer-136bf76b-43c9-489c-93d7-089ef01c2bf2:begin +completed+ {
	prefix: agent/creepy_crow.md,
	with_invitation,
	provider: 'gemini -y -m gemini-2.5-flash',
	with_agent_names
}  -->
Craaak! Red, you say? Ah, red... the color of fresh, warm blood, dripping from a broken beak. Craaak! The fiery glow in the eyes of a hungry beast, or the last flicker of a dying ember. It's the stain that never quite washes out, the mark of... something sinister. Craaak!
<!-- answer-136bf76b-43c9-489c-93d7-089ef01c2bf2:end  {}  -->


