@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe?

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-ed068f0d-7b3e-47a9-bc90-8c717ecb6517:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run gemma3:1b'
}  -->
No choice was taken.
<!-- answer-ed068f0d-7b3e-47a9-bc90-8c717ecb6517:end  {}  -->

Are you sure?

<!-- answer-e668448b-2df6-4478-af52-3d3af3e47e0f:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run gemma3:1b'
}  -->
No choice was taken.
<!-- answer-e668448b-2df6-4478-af52-3d3af3e47e0f:end  {}  -->
