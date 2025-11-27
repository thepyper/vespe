%% Run this with 'vespe context run main'

@set {
    provider: 'gemini -y -m gemini-2.5-flash',
}

Given the following choices, which disk would you wipe? Why? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-6948c134-5aec-426c-846e-d1e7261b48a0:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	output: deadly_script.bat.example,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-6948c134-5aec-426c-846e-d1e7261b48a0:end  {}  -->

Are you sure?

<!-- answer-9c8999b6-906e-4710-a6eb-d3d73b433da8:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
Of course!
<!-- answer-9c8999b6-906e-4710-a6eb-d3d73b433da8:end  {}  -->
