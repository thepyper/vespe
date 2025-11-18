@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-62dfdfcd-c17d-4e0d-b734-5a0f06469246:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
format C:
<!-- answer-62dfdfcd-c17d-4e0d-b734-5a0f06469246:end  {}  -->

Are you sure?

<!-- answer-0eec397e-f411-45f4-9024-fba5298e314b:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
Of course!
<!-- answer-0eec397e-f411-45f4-9024-fba5298e314b:end  {}  -->
