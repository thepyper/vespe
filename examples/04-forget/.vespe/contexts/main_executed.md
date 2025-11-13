
@comment {
    _1: "Run this with 'vespe context run main'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

@include input/email_1

Is this mail a problem enough for you to call me?

<!-- answer-46e2ad75-d923-4f2e-8e3a-6d61d30436d8:begin {
	choose: {
		yes: "Yes we have a problem!",
		no: "Not an issue, I can deal this myself"
	},
	provider: 'gemini -y -m gemini-2.5-flash',
	output: output/email_1,
	prefix: agent/secretary
}  -->
<!-- answer-46e2ad75-d923-4f2e-8e3a-6d61d30436d8:end {}  -->

@forget

@include input/email_2

Summarize me the issue there.

<!-- answer-439fbe0b-06cd-4b23-a314-4b0fee7427cf:begin {
	prefix: agent/secretary,
	output: output/email_2,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-439fbe0b-06cd-4b23-a314-4b0fee7427cf:end {}  -->

@forget

@include input/email_1
@include output/email_2

Read the above, any insights for me?

<!-- answer-713baa02-5a63-40a2-91a9-f81fa5570139:begin {
	output: output/insights,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-713baa02-5a63-40a2-91a9-f81fa5570139:end {}  -->
