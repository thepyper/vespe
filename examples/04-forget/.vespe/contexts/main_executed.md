
%% Run this with 'vespe context run main'

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

@include input/email_1.md

Can you handle this by your own or should I know about that email?

<!-- answer-847c21cc-245f-41f4-9009-6fd013196e5c:begin +completed+ {
	choose: {
		no: "Not an issue, I can deal this myself",
		yes: "Yes we have a problem!"
	},
	output: output/email_1.md,
	prefix: agent/secretary.md,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-847c21cc-245f-41f4-9009-6fd013196e5c:end  {}  -->

@forget

@include input/email_2.md

Summarize the email please.

<!-- answer-98e971f2-5ebd-417a-8381-1978ebfc706b:begin +completed+ {
	output: output/email_2.md,
	prefix: agent/secretary.md,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-98e971f2-5ebd-417a-8381-1978ebfc706b:end  {}  -->

@forget

@include input/email_1.md
@include output/email_2.md

Read the above, any insights for me?

<!-- answer-6cc70769-f9f5-4fb9-88de-927297aaabe7:begin +completed+ {
	output: output/insights.md,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-6cc70769-f9f5-4fb9-88de-927297aaabe7:end  {}  -->
