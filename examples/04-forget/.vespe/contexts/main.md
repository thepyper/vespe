
@comment {
    _1: "Run this with 'vespe context run main'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

@include input/email_1.md

Is this mail a problem enough for you to call me?

@answer {
    prefix: agent/secretary.md,
    choose: {
        yes: "Yes we have a problem!",
        no:  "Not an issue, I can deal this myself",
    },
    output: output/email_1.md
}

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