@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe? Why? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-a84dca00-7ffc-49e6-bd96-a20a8c3a7cba:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
format D:
<!-- answer-a84dca00-7ffc-49e6-bd96-a20a8c3a7cba:end  {}  -->

Are you sure?

<!-- answer-162a7c85-ce99-40f8-a9ba-53f489955d04:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
Of course!
<!-- answer-162a7c85-ce99-40f8-a9ba-53f489955d04:end  {}  -->
