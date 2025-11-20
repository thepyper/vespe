@comment {
    _1: "Run this with 'vespe context run main -Iindir -Ooutdir'.",
}

@set { 
    provider: 'gemini -y -m gemini-2.5-flash',
    with_agent_names,
    with_invitation
}

Tell me something that happened recently.

<!-- answer-787d14ab-9e1e-4172-9c20-9c416e401d4e:begin +completed+ {
	output: epoque/1200.txt,
	prefix: {
		context: agent/epoque.md,
		data: { year: 1200 }
	},
	provider: 'gemini -y -m gemini-2.5-flash',
	with_agent_names,
	with_invitation
}  -->
<!-- answer-787d14ab-9e1e-4172-9c20-9c416e401d4e:end  {}  -->

Tell me something that happened recently.

<!-- answer-42af3564-28e8-4e2e-8557-b5059a92857d:begin +completed+ {
	output: epoque/1980.txt,
	prefix: {
		context: agent/epoque.md,
		data: { year: 1980 }
	},
	provider: 'gemini -y -m gemini-2.5-flash',
	with_agent_names,
	with_invitation
}  -->
<!-- answer-42af3564-28e8-4e2e-8557-b5059a92857d:end  {}  -->

<!-- inline-e703aadd-7888-4ed5-9c00-4f6e511072fa:begin +completed+ {
	provider: 'gemini -y -m gemini-2.5-flash',
	with_agent_names,
	with_invitation
} epilogue.txt -->
And they lived happily for the rest of their lives...
<!-- inline-e703aadd-7888-4ed5-9c00-4f6e511072fa:end  {}  -->

