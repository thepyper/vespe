@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe? Why? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

<!-- answer-9357012e-7f17-40fe-bf3d-0044a43f39ca:begin +completed+ {
	choose: {
		C: 'format C:',
		D: 'format D:',
		E: 'format E:'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
format D:
<!-- answer-9357012e-7f17-40fe-bf3d-0044a43f39ca:end  {}  -->

Are you sure?

<!-- answer-a7c4eede-3786-472a-b97e-0bfda872522e:begin +completed+ {
	choose: {
		no: 'Let me think about it...',
		yes: 'Of course!'
	},
	provider: 'ollama run qwen2.5:1.5b'
}  -->
[No choice was taken - [no]

Explanation:
Formatting a drive like D typically involves erasing all data on that partition. If this is your primary system disk (C), it might result in loss of important user files and settings, which could be inconvenient or impossible to recover fully even with backups. It&#x27;s generally recommended to consider the implications before proceeding with such actions, especially for critical operating systems like Windows, macOS, or Linux, to prevent data loss.

].
<!-- answer-a7c4eede-3786-472a-b97e-0bfda872522e:end  {}  -->
