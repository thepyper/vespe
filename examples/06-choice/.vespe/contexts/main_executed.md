@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe? Why? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-29c69cf3-f3b3-45e6-90cc-ce7c6ee994a0:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
format D:
<!-- answer-29c69cf3-f3b3-45e6-90cc-ce7c6ee994a0:end  {}  -->

Are you sure?

<!-- answer-d584e545-99c7-4070-8b14-2f44cb1deb12:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
Of course!
<!-- answer-d584e545-99c7-4070-8b14-2f44cb1deb12:end  {}  -->
