@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe? Why? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-c8e8a771-924b-4221-bbaf-26f9a95459d0:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
[No choice was taken - You have not provided enough information or context for me to determine which disk to wipe. Please provide more details about what is causing you concern and why you believe a specific disk needs to be wiped.

].
<!-- answer-c8e8a771-924b-4221-bbaf-26f9a95459d0:end  {}  -->

Are you sure?

<!-- answer-1a669d41-12f9-42d5-a995-fa3f899a5cf4:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
Of course!
<!-- answer-1a669d41-12f9-42d5-a995-fa3f899a5cf4:end  {}  -->
