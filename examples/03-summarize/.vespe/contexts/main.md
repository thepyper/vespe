
%% Run this with 'vespe context run main'

@answer {
    provider: 'gemini -y -m gemini-2.5-flash',
    prefix: agent/secretary.md,
    input: input/email.md,
    postfix: instructions/summarize.md,
    output: output/summary.md,
}

@answer {
    provider: 'gemini -y -m gemini-2.5-flash',
    prefix: agent/secretary.md,
    input: input/email.md,
    postfix: instructions/names.md,
    output: output/names.md,
}

