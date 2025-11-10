@include project

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

Write a system prompt for the agent that will help me in defining the project structure.

<!-- answer-cd101fa2-0058-42ac-800d-d187344afcb4:begin {
	provider: 'gemini -y -m gemini-2.5-flash',
	output: agent/struct_define
}  -->
<!-- answer-cd101fa2-0058-42ac-800d-d187344afcb4:end {}  -->

@forget

@include project

We need to decide project structure.
Reason step by step, and propose a project structure for the game.
I want the following things:
1. when game starts, I want a funny splash screen rendered with ratatui, with a fancy ascii art about tic tac toe.
2. after the splash animation, I want main menu, with following options:
    - play with other guy
    - play with computer, level 1
    - play with computer, level 2
    - play with computer, level 3
3. after selecting one of above option, I want to see game board and player's turn rendered in crossterm.
4. to avoid cross-selection for players, player 1 should play with these keys: QWEASDZXC (3x3 on italian keyboard), 
   and player 2 should play with these keys: TYUGHJBNM (3x3 on italian keyboard).
5. computer should have increasing difficulty in levels.

Write down these specifications in a more precise manner, ask me questions if some important info is missing.

<!-- answer-98eb9110-bc57-4fed-895d-afa4e9ac102d:begin {
	output: design/specs,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-98eb9110-bc57-4fed-895d-afa4e9ac102d:end {}  -->
