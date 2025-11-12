
@comment {
    _1: "Run this with 'ctx context run main'.",
}

@answer {
    provider: 'gemini -y -m gemini-2.5-flash',
    prefix: agent/secretary,
    input: input/email,
    postfix: instructions/summarize,
    output: output/summary,
}

@answer {
    provider: 'gemini -y -m gemini-2.5-flash',
    prefix: agent/secretary,
    input: input/email,
    postfix: instructions/names,
    output: output/names,
}

