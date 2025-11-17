
@comment {
    _1: "Run this with 'echo violet | vespe context run main sepia yellow -Dmy_var=green'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

<!-- inline-7269fb00-a612-41fa-a4da-7fabdfa0165b:begin +completed+ {
	data: { color: 'blue' },
	provider: 'gemini -y -m gemini-2.5-flash'
} template/about_color.md -->
Tell me 5 trivia about color blue.
<!-- answer-47e66216-8d03-4b4f-8948-90356f4ec679:begin +need_processing+ { provider: 'gemini -y -m gemini-2.5-flash' }  -->
<!-- answer-47e66216-8d03-4b4f-8948-90356f4ec679:end  {}  -->


<!-- inline-7269fb00-a612-41fa-a4da-7fabdfa0165b:end  {}  -->

@inline { data: { color: 'orange' } } template/about_color.md

@inline { data: { color: 'sepia' } } template/about_color.md

@inline { data: { color: 'sepia yellow' } } template/about_color.md

@inline { data: { color: 'violet
' } } template/about_color.md

@inline { data: { color: 'green' } } template/about_color.md

