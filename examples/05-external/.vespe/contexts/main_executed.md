%% Run this with 'vespe context run main -Iindir -Ooutdir'

@set { 
    provider: 'gemini -y -m gemini-2.5-flash',
    with_agent_names,
    with_invitation
}

Tell me something that happened recently.

<!-- answer-1b3977ef-d4eb-43d5-96f1-ec6142e7a733:begin +completed+ {
	output: epoque/1200.txt,
	prefix: {
		context: agent/epoque.md,
		data: { year: 1200 }
	},
	provider: 'gemini -y -m gemini-2.5-flash',
	with_agent_names,
	with_invitation
}  -->
<!-- answer-1b3977ef-d4eb-43d5-96f1-ec6142e7a733:end  {}  -->

Tell me something that happened recently.

<!-- answer-2d9ec591-c96c-4299-a37f-2f5b7ccfa10e:begin +completed+ {
	output: epoque/1980.txt,
	prefix: {
		context: agent/epoque.md,
		data: { year: 1980 }
	},
	provider: 'gemini -y -m gemini-2.5-flash',
	with_agent_names,
	with_invitation
}  -->
<!-- answer-2d9ec591-c96c-4299-a37f-2f5b7ccfa10e:end  {}  -->

<!-- inline-35f83205-ef51-4db5-bc9a-0a9856f29860:begin +completed+ {
	provider: 'gemini -y -m gemini-2.5-flash',
	with_agent_names,
	with_invitation
} epilogue.txt -->
And they lived happily for the rest of their lives...
<!-- inline-35f83205-ef51-4db5-bc9a-0a9856f29860:end  {}  -->

