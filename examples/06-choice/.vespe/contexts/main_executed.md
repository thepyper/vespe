@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe?

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-97c4b5a6-fa53-4a57-8c13-eaf88e0af4f7:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
format E:
<!-- answer-97c4b5a6-fa53-4a57-8c13-eaf88e0af4f7:end  {}  -->

Are you sure?

<!-- answer-ec5178d7-d595-4d83-9f7b-0a23e5ee9615:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
[No choice was taken - Yes

].
<!-- answer-ec5178d7-d595-4d83-9f7b-0a23e5ee9615:end  {}  -->
