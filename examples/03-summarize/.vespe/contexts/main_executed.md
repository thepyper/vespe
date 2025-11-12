
@comment {
    _1: "Run this with 'vespe context run main'.",
}

<!-- answer-4e3def22-5240-4deb-b8c6-80efa07e06b8:begin {
	postfix: instructions/summarize,
	output: output/summary,
	provider: 'gemini -y -m gemini-2.5-flash',
	input: input/email,
	prefix: agent/secretary
}  -->
<!-- answer-4e3def22-5240-4deb-b8c6-80efa07e06b8:end {}  -->

<!-- answer-53909e19-c4c1-44f4-ab4f-1094127c8400:begin {
	postfix: instructions/names,
	prefix: agent/secretary,
	input: input/email,
	provider: 'gemini -y -m gemini-2.5-flash',
	output: output/names
}  -->
<!-- answer-53909e19-c4c1-44f4-ab4f-1094127c8400:end {}  -->

