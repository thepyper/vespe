
@comment {
    _1: "Run this with 'vespe context run main'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

@include input/email_1

Is this mail a problem enough for you to call me?

<!-- answer-56bcff4f-760b-44f0-b050-7d007f3ef353:begin {
	provider: 'gemini -y -m gemini-2.5-flash',
	choose: {
		no: "Not an issue, I can deal this myself",
		yes: "Yes we have a problem!"
	},
	prefix: agent/secretary,
	output: output/email_1
}  -->
<!-- answer-56bcff4f-760b-44f0-b050-7d007f3ef353:end {}  -->

@forget

@include input/email_2

Summarize me the issue there.

<!-- answer-5f6116a1-8bcd-4ed9-afb7-d2b24351c53b:begin {
	output: output/email_2,
	provider: 'gemini -y -m gemini-2.5-flash',
	prefix: agent/secretary
}  -->
<!-- answer-5f6116a1-8bcd-4ed9-afb7-d2b24351c53b:end {}  -->

@forget

@include input/email_1
@include output/email_2

Read the above, any insights for me?

@answer {
    output: output/insights
}