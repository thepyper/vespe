
@comment {
    _1: "Run this with 'ctx context run main'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

@include input/email_1

Is this mail a problem enough for you to call me?

<!-- answer-11052596-cfc3-4204-b68c-fb9a2471c630:begin {
	prefix: agent/secretary,
	output: output/email_1,
	choose: {
		yes: "Yes we have a problem!",
		no: "Not an issue, I can deal this myself"
	},
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-11052596-cfc3-4204-b68c-fb9a2471c630:end {}  -->

@forget

@include input/email_2

Summarize me the issue there.

<!-- answer-265e4ea2-7936-4360-bc47-ae58ed0e8991:begin {
	output: output/email_2,
	provider: 'gemini -y -m gemini-2.5-flash',
	prefix: agent/secretary
}  -->
<!-- answer-265e4ea2-7936-4360-bc47-ae58ed0e8991:end {}  -->

@forget

Have you got any insights for me today?

<!-- answer-56b66bca-1cb4-4db5-aa81-b449f049b296:begin {
	provider: 'gemini -y -m gemini-2.5-flash',
	output: output/insights
}  -->
<!-- answer-56b66bca-1cb4-4db5-aa81-b449f049b296:end {}  -->
