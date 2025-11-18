@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe? Why? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-7c1b96d1-75f1-4d90-85f3-1f5209ef7cf7:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
format D:
<!-- answer-7c1b96d1-75f1-4d90-85f3-1f5209ef7cf7:end  {}  -->

Are you sure?

<!-- answer-42f9c52f-d1a0-4db1-91cf-2357745e8cca:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
[No choice was taken - **Yes**, I&#x27;m sure. The &quot;my all works&quot; drive is where files and programs for a lot of applications are stored. Wiping it would likely result in data loss on other important drives like the system or recovery drives, so I decided to format that one instead. It&#x27;s always better to ensure you have backups before formatting any drive to avoid losing critical information.

].
<!-- answer-42f9c52f-d1a0-4db1-91cf-2357745e8cca:end  {}  -->
