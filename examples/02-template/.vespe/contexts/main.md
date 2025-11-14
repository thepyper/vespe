
@comment {
    _1: "Run this with 'echo violet | vespe context run main sepia yellow'.",
}

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

@inline { data: { color: 'blue' } } template/about_color.md

@inline { data: { color: 'orange' } } template/about_color.md

@inline { data: { color: '{{$1}}' } } template/about_color.md

@inline { data: { color: '{{$args}}' } } template/about_color.md

@inline { data: { color: '{{$input}}' } } template/about_color.md

