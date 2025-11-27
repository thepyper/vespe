%% Run this with 'vespe context run main -Iindir -Ooutdir'

@set { 
    provider: 'gemini -y -m gemini-2.5-flash',
    with_agent_names,
    with_invitation
}

Tell me something that happened recently.

@answer { prefix: { context: agent/epoque.md, data: { year: 1200 } }, output: epoque/1200.txt }

Tell me something that happened recently.

@answer { prefix: { context: agent/epoque.md, data: { year: 1980 } }, output: epoque/1980.txt }

@inline epilogue.txt

