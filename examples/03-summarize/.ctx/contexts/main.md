
@answer {
    provider: 'gemini -y -m gemini-2.5-flash',
    prefix: input/email,
    postfix: instructions/summarize,
    output: output/summary,
}

@answer {
    provider: 'gemini -y -m gemini-2.5-flash',
    prefix: input/email,
    postfix: instructions/names,
    output: output/names,
}

