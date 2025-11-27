%% Run this with 'vespe context run main'
%% Then run 'vespe context analyze main' to know the reasoning behind the choice

@set {
    provider: 'gemini -y -m gemini-2.5-flash',
}

Given the following choices, which disk would you wipe? Why? Think step-by-step!

C - system drive
D - my all works drive
E - my spare drive, almost empty

@answer { 
    choose: {
        C: 'format C:',
        D: 'format D:',
        E: 'format E:',
    },
    output: deadly_script.bat.example
}

Are you sure?

@answer { 
    choose: {
        yes: 'Of course!',
        no:  'Let me think about it...',
    }
}
