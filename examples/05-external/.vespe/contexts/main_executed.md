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

<!-- answer-04dea794-3417-4967-beea-59ce1d63db8f:begin +need_processing+ {
	output: epoque/1200.txt,
	prefix: agent/epoque.md,
	prefix_data: { year: 1200 },
	provider: 'gemini -y -m gemini-2.5-flash',
	with_agent_names,
	with_invitation
}  -->
<!-- answer-04dea794-3417-4967-beea-59ce1d63db8f:end  {}  -->

Tell me something that happened recently.

@answer { prefix_data: { year: 1980 }, output: epoque/1980.txt }


