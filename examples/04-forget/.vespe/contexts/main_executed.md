
@comment {
    _1: "Run this with 'vespe context run main'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

@include input/email_1.md

Is this mail a problem enough for you to call me?

<!-- answer-589e178a-c2aa-4f2c-97e1-7406f427ce83:begin +completed+ {
	prefix: agent/secretary.md,
	choose: {
		yes: "Yes we have a problem!",
		no: "Not an issue, I can deal this myself"
	},
	provider: 'gemini -y -m gemini-2.5-flash',
	output: output/email_1.md
}  -->
<!-- answer-589e178a-c2aa-4f2c-97e1-7406f427ce83:end  {}  -->

@forget

@include input/email_2.md

Summarize me the issue there.

<!-- answer-5e01d3f6-dccc-4de5-ad88-2bb3c5415131:begin +completed+ {
	output: output/email_2.md,
	provider: 'gemini -y -m gemini-2.5-flash',
	prefix: agent/secretary.md
}  -->
<!-- answer-5e01d3f6-dccc-4de5-ad88-2bb3c5415131:end  {}  -->

@forget

@include input/email_1.md
@include output/email_2.md

Read the above, any insights for me?

<!-- answer-c9ffa703-3ca5-4576-ae34-fde53f1d904d:begin +completed+ {
	output: output/insights.md,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-c9ffa703-3ca5-4576-ae34-fde53f1d904d:end  {}  -->
