
@comment {
    _1: "Run this with 'vespe context run main'.",
}

<!-- answer-b3159f99-db9c-46af-af08-516c07cafa13:begin {
	provider: 'gemini -y -m gemini-2.5-flash',
	prefix: agent/secretary,
	input: input/email,
	postfix: instructions/summarize,
	output: output/summary
}  -->
<!-- answer-b3159f99-db9c-46af-af08-516c07cafa13:end {}  -->

<!-- answer-0178162b-42d6-4810-81c5-da1dfbd77622:begin {
	input: input/email,
	output: output/names,
	provider: 'gemini -y -m gemini-2.5-flash',
	postfix: instructions/names,
	prefix: agent/secretary
}  -->
<!-- answer-0178162b-42d6-4810-81c5-da1dfbd77622:end {}  -->

