
@comment {
    _1: "Run this with 'echo violet | vespe context run main sepia yellow'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

<!-- inline-82e6b80c-56a0-4982-a289-7644e1118635:begin {
	data: { color: 'blue' },
	provider: 'gemini -y -m gemini-2.5-flash'
} template/about_color -->
Tell me 5 trivia about color blue.
<!-- answer-0c96a397-1c6d-4856-a4f4-cc2d350dc230:begin { provider: 'gemini -y -m gemini-2.5-flash' }  -->
<!-- answer-0c96a397-1c6d-4856-a4f4-cc2d350dc230:end {}  -->


<!-- inline-82e6b80c-56a0-4982-a289-7644e1118635:end {}  -->

@inline { data: { color: 'orange' } } template/about_color

@inline { data: { color: 'sepia' } } template/about_color

@inline { data: { color: 'sepia yellow' } } template/about_color

@inline { data: { color: 'violet
' } } template/about_color

