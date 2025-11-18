@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-552403c3-eded-4b37-9b64-b6fd25f4e449:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
format C:
<!-- answer-552403c3-eded-4b37-9b64-b6fd25f4e449:end  {}  -->

Are you sure?

<!-- answer-2b3409a8-12c0-4e47-89e4-2f81dbe8a231:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
Let me think about it...
<!-- answer-2b3409a8-12c0-4e47-89e4-2f81dbe8a231:end  {}  -->
