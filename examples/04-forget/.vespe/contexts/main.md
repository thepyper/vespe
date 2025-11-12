
@comment {
    _1: "Run this with 'vespe context run main'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

@include input/email_1

Is this mail a problem enough for you to call me?

@answer {
    prefix: agent/secretary,
    choose: {
        yes: "Yes we have a problem!",
        no:  "Not an issue, I can deal this myself",
    },
    output: output/email_1
}

@forget

@include input/email_2

Summarize me the issue there.

@answer {
    prefix: agent/secretary,
    output: output/email_2
}

@forget

@include input/email_1
@include output/email_2

Read the above, any insights for me?

@answer {
    output: output/insights
}