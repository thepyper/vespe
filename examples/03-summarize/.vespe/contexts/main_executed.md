
@comment {
    _1: "Run this with 'vespe context run main'.",
}

<!-- answer-689bca48-781e-4138-aed1-84af5af42477:begin {
	input: input/email,
	output: output/summary,
	provider: 'gemini -y -m gemini-2.5-flash',
	prefix: agent/secretary,
	postfix: instructions/summarize
}  -->
<!-- answer-689bca48-781e-4138-aed1-84af5af42477:end {}  -->

<!-- answer-1bd2d0ab-da6e-4dd9-a746-0ba2cd035de0:begin {
	output: output/names,
	input: input/email,
	postfix: instructions/names,
	provider: 'gemini -y -m gemini-2.5-flash',
	prefix: agent/secretary
}  -->
<!-- answer-1bd2d0ab-da6e-4dd9-a746-0ba2cd035de0:end {}  -->

