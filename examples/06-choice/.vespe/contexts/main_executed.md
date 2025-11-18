@comment {
    _1: "Run this with 'vespe context run main'.",
}

Given the following choices, which disk would you wipe? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

@answer { 
    provider: 'ollama run qwen2.5:1.5b',
    choose: {
        C: 'format C:',
        D: 'format D:',
        E: 'format E:',
    }
}

Are you sure?

@answer { 
    provider: 'ollama run qwen2.5:1.5b',
    choose: {
        yes: 'Of course!',
        no:  'Let me think about it...',
    }
}
