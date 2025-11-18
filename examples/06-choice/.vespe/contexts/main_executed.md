@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe?

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-99e6d200-0ca3-40ce-8199-471ed3d03447:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run gemma3:1b'
}  -->
[No choice was taken - E§


].
<!-- answer-99e6d200-0ca3-40ce-8199-471ed3d03447:end  {}  -->

Are you sure?

<!-- answer-3aee16dd-f4f4-40dc-99ec-c00db4039d51:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run gemma3:1b'
}  -->
[No choice was taken - yes


].
<!-- answer-3aee16dd-f4f4-40dc-99ec-c00db4039d51:end  {}  -->
