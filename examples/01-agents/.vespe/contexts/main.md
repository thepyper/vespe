%% Run this with 'vespe context run main'

@set {
	with_agent_names,
	with_invitation,
}

@include agent/gemini_25_flash_yolo.md

Tell me something about blue.

@answer { prefix: agent/funny_clown.md }

Tell me something about red.

@answer { prefix: agent/creepy_crow.md }


