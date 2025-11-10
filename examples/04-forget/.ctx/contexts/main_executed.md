
@comment {
    _1: "Run this with 'ctx context run main'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

@include input/email_1

Is this mail a problem enough for you to call me?

<!-- answer-ab215a20-a8a6-4bfe-acb3-07e303029a92:begin {
	output: output/email_1,
	choose: {
		no: "Not an issue, I can deal this myself",
		yes: "Yes we have a problem!"
	},
	prefix: agent/secretary,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-ab215a20-a8a6-4bfe-acb3-07e303029a92:end {}  -->

@forget

@include input/email_2

Summarize me the issue there.

<!-- answer-db985a43-ab59-4bb1-b896-fe6390b10f83:begin {
	prefix: agent/secretary,
	output: output/email_2,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-db985a43-ab59-4bb1-b896-fe6390b10f83:end {}  -->

@forget

@include input/email_1
@include output/email_2

Read the above, any insights for me?

<!-- answer-d209ba51-3439-4afd-95f8-9b3b2df59e31:begin {
	output: output/insights,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-d209ba51-3439-4afd-95f8-9b3b2df59e31:end {}  -->
