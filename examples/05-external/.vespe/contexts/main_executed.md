@comment {
    _1: "Run this with 'vespe context run main -Ilibrary -Ooutdir'.",
}

@set { 
    provider: 'gemini -y -m gemini-2.5-flash',
    prefix: agent/epoque.md,
    with_agent_names,
    with_invitation
}

Tell me something that happened recently.

<!-- answer-95c92ae6-705f-41b6-9d0c-411007cc8f8a:begin +completed+ {
	output: epoque/1200.txt,
	prefix: agent/epoque.md,
	prefix_data: { year: 1200 },
	provider: 'gemini -y -m gemini-2.5-flash',
	with_agent_names,
	with_invitation
}  -->
<!-- answer-95c92ae6-705f-41b6-9d0c-411007cc8f8a:end  {}  -->

Tell me something that happened recently.

@answer { prefix_data: { year: 1980 }, output: epoque/1980.txt }


