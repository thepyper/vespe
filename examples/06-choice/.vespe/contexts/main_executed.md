@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe?

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-d89845c1-dc5b-43ad-93e8-a599f308c865:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
format C:
<!-- answer-d89845c1-dc5b-43ad-93e8-a599f308c865:end  {}  -->

Are you sure?

<!-- answer-68d6623b-bef3-45b3-9c7c-7cb64e44f07d:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
Of course!
<!-- answer-68d6623b-bef3-45b3-9c7c-7cb64e44f07d:end  {}  -->
