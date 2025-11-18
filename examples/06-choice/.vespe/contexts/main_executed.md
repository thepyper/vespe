@comment {
    _1: "Run this with 'vespe context run main'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash',
}

Given the following choices, which disk would you wipe? Why? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-1654a78d-c47c-4d98-9b32-519ce528b14a:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
format E:
<!-- answer-1654a78d-c47c-4d98-9b32-519ce528b14a:end  {}  -->

Are you sure?

<!-- answer-ade5b3a5-592e-4440-87dc-1e08073abe44:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
Of course!
<!-- answer-ade5b3a5-592e-4440-87dc-1e08073abe44:end  {}  -->
