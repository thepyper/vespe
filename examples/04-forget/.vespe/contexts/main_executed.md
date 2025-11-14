
@comment {
    _1: "Run this with 'vespe context run main'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

@include input/email_1.md

Is this mail a problem enough for you to call me?

<!-- answer-589e178a-c2aa-4f2c-97e1-7406f427ce83:begin +need_processing+ {
	prefix: agent/secretary.md,
	output: output/email_1.md,
	provider: 'gemini -y -m gemini-2.5-flash',
	choose: {
		no: "Not an issue, I can deal this myself",
		yes: "Yes we have a problem!"
	}
}  -->
<!-- answer-589e178a-c2aa-4f2c-97e1-7406f427ce83:end  {}  -->

@forget

@include input/email_2.md

Summarize me the issue there.

@answer {
    prefix: agent/secretary.md,
    output: output/email_2.md
}

@forget

@include input/email_1.md
@include output/email_2.md

Read the above, any insights for me?

@answer {
    output: output/insights.md
}